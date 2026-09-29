<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 4: Storage Drivers & Virtual Filesystems

Milestone 4 addresses persistent mass storage, hierarchical filesystem namespaces, and low-overhead block caching from raw AHCI/NVMe hardware registers up to POSIX VFS system calls.

---

## 1. Storage & Filesystem Layered Pipeline

```mermaid
graph TD
    App["Application / Shell (VFS Calls)"] --> VFS["Virtual Filesystem Layer (VFS)<br/><i>crates/fs/src/vfs/</i>"]
    VFS --> MountTable["Mount Table Dispatcher (/system, /data, /config, /apps)"]
    MountTable --> EXT4["Native EXT4 Engine (Extent Tree Parser)"]
    MountTable --> FAT["FAT12/16/32 Driver (Cluster Chain Engine)"]
    MountTable --> USTAR["USTAR Ramdisk Driver (Boot Initrd)"]
    EXT4 & FAT --> LRUCache["16-Slot Sector LRU Cache Engine"]
    LRUCache --> BlockDev["Unified Block Device Abstraction"]
    BlockDev --> AHCI["AHCI SATA Controller Driver (FIS & PRDT)"]
    BlockDev --> NVMe["NVMe Controller Driver (Admin Queue Depth: 64)"]
    BlockDev --> ATA["Legacy ATA / IDE PIO Driver"]
```

---

## 2. Core Engineering Implementation Details

### A. AHCI SATA Driver & Frame Information Structures (FIS)
Keira interacts with modern SATA storage via the Advanced Host Controller Interface (AHCI):
- Discovers the AHCI controller via PCI Class `0x01` (Mass Storage), Subclass `0x06` (SATA), Prog IF `0x01` (AHCI 1.0).
- Maps the Generic Host Control registers (ABAR MMIO space).
- Configures Command List and Received FIS (Frame Information Structure) buffers in physical DMA memory.
- Uses **Physical Region Descriptor Tables (PRDT)** to stream sectors directly between disk controllers and kernel RAM without CPU polling loops.

### B. Virtual Filesystem (VFS) Architecture
The VFS abstracts all storage media behind a unified POSIX tree:
- **Canonical 5-Directory Hierarchy**: `/system` (core binaries/lib), `/apps` (userland binaries), `/config` (boot and sys config), `/data` (logs and persistent data), and `/temp` (scratch workspace).
- **Mount Points**: Block devices and ramdisks are dynamically bound to root subdirectories via `vfs::mount(target_path, driver)`.
- **Path Resolution**: The `resolve_path()` engine parses absolute and relative paths, resolves directory symlinks, and dispatches file operations (`open`, `read`, `write`, `close`, `seek`) to the target filesystem driver.

### C. Native Linux EXT4 Extent Tree Traversal
Rather than relying on outdated indirect block pointers, Keira implements a bare-metal reader for Linux EXT4 filesystems:
1. Validates the **Superblock Magic** signature `0xEF53`.
2. Reads the Block Group Descriptor Table to locate inode tables and bitmaps.
3. Parses the 4-level **Extent Tree** embedded within the inode's `i_block` structure:
   - `ext4_extent_header`: Magic `0xF30A`, entry count, and depth.
   - `ext4_extent_idx`: Points to intermediate extent nodes down the tree.
   - `ext4_extent`: Leaf node specifying file block offset, block length, and starting physical block address.
4. Allows direct, high-performance streaming of multi-megabyte kernel assets from standard Linux partitions.

### D. Sector LRU Cache Engine
Disk access latency is mitigated by an in-kernel **Least Recently Used (LRU) Sector Cache**:
- Maintains 16 slots of 512-byte sector buffers.
- Track hits, misses, and evictions to maximize throughput.
- Flushed synchronously during `sync` commands to ensure persistent integrity.

---

## 3. Real-Time Telemetry & Shell Verification

```text
keira:/system# drives
NAME       TYPE       SIZE (KB)   STATUS
----       ----       ---------   ------
ahci0      SATA Disk 10240        [Mounted]

keira:/system# disk
Active Drive (ahci0) Size: 10 MB (20480 sectors)
Filesystem:    FAT16
Cluster Size:  2048 bytes (4 sectors)
Reserved Secs: 4
Root Directory: 512 entries (start sector: 132)
LRU Cache:     5/16 slots (Hits: 1, Misses: 5, Evictions: 0, Hit Ratio: 16%)

keira:/system# list /config
Directory of IDE disk:
  [dir]  boot
  [dir]  sys

keira:/system# view /config/boot/grub.cfg
console=tty0 serial=ttyS0,115200 root=/dev/sda1 quiet loglevel=3
```
