<!-- SPDX-License-Identifier: GPL-2.0-only -->

# VFS File Operations, Path Resolution & Dentry Cache

The Virtual Filesystem (VFS) provides a unified abstraction across registered storage providers (FAT, EXT4, Tmpfs, ProcFS and DevFS).

---

## 1. VFS Dentry Cache (`dcache`) Architecture

Inspired by the Linux kernel (`fs/dcache.c`), Keira incorporates a resident, in-memory directory entry cache (`DentryCache`) located in `crates/fs/src/cache/dentry.rs`:

- **$O(1)$ Fast-Path Lookups**: Maps canonical paths directly to cached metadata (filesystem type, inode, file size and directory flag) via 32-bit FNV-1a hashing.
- **Negative Dentry Caching**: Confirmed nonexistent paths are recorded as negative dentries (`DENTRY_FLAG_NEGATIVE`), eliminating expensive disk and driver traversals for missing files.
- **LRU Second-Chance Eviction**: Maintains an access clock and referenced flags to preserve frequently accessed working sets across execution lifecycles.
- **Cache Invalidation Hooks**: Directory and file mutations (`create_file`, `create_dir`, `remove_entry` and `write_file`) automatically invalidate matching path entries and descendant hierarchy prefixes.

---

## 2. VFS Inode Cache (`icache`) Architecture

Inspired by the Linux kernel (`fs/inode.c`), Keira incorporates a resident, in-memory inode descriptor cache (`CachedInode`) located in `crates/fs/src/cache/inode.rs`:

- **$O(1)$ Inode Descriptor Resolution**: Resolves inode metadata (mode, uid, gid, size, timestamps and nlink) directly from memory by `(device, inode)` tuple.
- **Dirty State Tracking**: Inodes modified during write operations are flagged with `INODE_FLAG_DIRTY`, deferring costly disk metadata updates until explicit flush or synchronization.
- **Storage Writeback Integration**: System-wide `sync` calls and `icache -f` automatically flush all dirty inodes to underlying storage media.
- **Second-Chance LRU CLOCK Eviction**: Utilizes an access sequence counter and referenced flags to reclaim clean victim slots under memory pressure.

---

## 3. Supervisor Inspection (`dcache` & `icache` commands)

The Ring 0 supervisor console provides diagnostic tools to inspect real-time cache efficiency:

```text
keira:/# dcache
VFS Dentry Cache (dcache) Telemetry:
  Total Capacity:     64 slots
  Active Entries:     18 (positive: 15, negative: 3)
  Memory Footprint:   2304 bytes
  Cache Hits:         142 (negative hits: 12)
  Cache Misses:       18
  LRU Evictions:      0
  Lookup Hit Ratio:   89%

keira:/# icache
VFS Inode Cache (icache) Telemetry:
  Total Capacity:     64 slots
  Active Inodes:      12 descriptors
  Dirty Inodes:       0 uncommitted
  Cache Hits:         128 lookups
  Cache Misses:       12 lookups
  Hit Ratio:          91%
  Clock Evictions:    0 slots
  Dirty Writebacks:   4 synchronizations
  Resident Memory:    3072 bytes
```

---

## 4. Standard File Operations

Standard VFS dispatchers:
- Node creation: `create_file()`, `create_dir()`, `create_symlink()`
- Existence and metadata: `exists()`, `is_dir()`, `get_file_size()`, `get_metadata()`
- Data transfer: `read_file()`, `read_file_offset()`, `write_file()`, `write_file_offset()`
- Mutation and removal: `set_permissions()`, `set_ownership()`, `remove_entry()`, `remove_symlink()`

