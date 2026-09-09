<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device & Hardware Shell Commands

This document details all native commands in Keira Kernel related to hardware diagnostics, block device control, device drivers, and virtualization interfaces.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `devices` | `devices` | `[Active]` | List registered character and block device nodes under `/system/dev/` and scan PCI bus |
| `drivers` | `drivers` | `[Active]` | Display loaded device driver descriptors and status (`Active` / `Idle`) |
| `framebuffer` | `framebuffer` | `[Active]` | Display active VGA/VBE graphical framebuffer resolution, pitch, and BPP |
| `usb` | `usb [list \| tree \| ports]` | `[Active]` | Enumerate USB host controllers, root hubs, and connected HID/storage devices |
| `epoll` | `epoll [status \| list \| create \| ctl \| wait \| test]` | `[Active]` | Inspect and manage `epoll` scalable I/O event multiplexer instances (Syscall 55, 56 & 57) |
| `kvm` | `kvm [status \| list \| create \| run <vm_id> <vcpu_id> \| vcpu <vm_id> <vcpu_id> \| test]` | `[Active]` | Kernel-based Virtual Machine hardware virtualization control and vCPU execution (Syscall 42 & 43) |
| `lkm` | `lkm [status \| lsmod \| list \| load <name> [size] \| unload <name> \| symbols \| test]` | `[Active]` | Loadable Kernel Module lifecycle management and dynamic symbol table (Syscall 34 & 35) |
| `lvm` | `lvm [status \| list \| info \| create \| lvcreate \| test]` | `[Active]` | Display and configure Logical Volume Manager volume groups (Syscall 74) |
| `nvme` | `nvme [status \| list \| identify \| namespaces \| test]` | `[Active]` | Display NVMe 1.4 PCIe controller registers, queues, and namespace stats |
| `raid` | `raid [status \| list \| sync \| rebuild \| test]` | `[Active]` | Inspect and synchronize Software RAID 0 (striping) and RAID 1 (mirroring) arrays (Syscall 74) |
| `swap` | `swap [status \| on [dev] \| off \| test]` | `[Active]` | Display active swap spaces, allocate slots, and configure backing devices |

---

## Detailed Usage

### `devices`
Enumerates all device nodes exposed by the Virtual Filesystem layer and scans the hardware PCI bus:
```bash
keira> devices
  [char]  /system/dev/console  (80x25 VGA / Serial)
  [char]  /system/dev/null     (Null sink)
  [char]  /system/dev/zero     (Zero stream)
  [char]  /system/dev/random   (Cryptographic RNG)
  [char]  /system/dev/ptmx     (Pseudo-terminal multiplexer)
  [block] /system/dev/sda      (Primary Block Disk)
  [block] /system/dev/sda1     (FAT16 Partition)
```

### `epoll`
Inspects and exercises the kernel scalable I/O event multiplexer subsystem:
```bash
keira> epoll status
Epoll Scalable I/O Multiplexer Engine (Syscall 55, 56 & 57) [Active]
  Status      : Operational (O(1) Ready Polling)
  Instances   : 1 active (Capacity: 8)
  Watched FDs : 2 descriptors in interest list
  Features    : Edge-Triggered (EPOLLET), EPOLLIN, EPOLLOUT, EPOLLERR, EPOLLHUP
  Syscalls    : 55 (epoll_create), 56 (epoll_ctl), 57 (epoll_wait)

keira> epoll list
Active Epoll Instances & Interest Lists:
  EPFD #1 (Size Hint: 32, Monitored FDs: 2, Total Polls: 1)
    -> FD #5 [Events: 0x0001 IN] Data: 0x100
    -> FD #6 [Events: 0x0005 IN OUT] Data: 0x200
```

### `framebuffer`
Queries hardware VBE graphical framebuffer capabilities:
```bash
keira> framebuffer
VGA / VBE Framebuffer Status:
  Resolution : 1024x768
  Color Depth: 32 bpp (ARGB8888)
  Pitch      : 4096 bytes/line
  Base Addr  : 0xFD000000
```

### `lvm`
Displays and manipulates Logical Volume Management volume groups and logical partitions:
```bash
keira> lvm status
Logical Volume Manager (LVM) [Active]
  Status      : Operational
  Volume Groups: 1 active (Capacity: 2)
  Total Storage: 64 MB (Free: 32 MB)
  Logical Vols: 2 mapped block devices
  Syscall     : 74 (SYS_RAID_LVM)

keira> lvm list
Volume Group vg_keira0 (64MB Total, 32MB Free) [2 PVs]:
  -> /dev/vg_keira0/lv_root (16MB, fat16) [ACTIVE]
  -> /dev/vg_keira0/lv_data (16MB, ext4) [ACTIVE]
```

### `nvme`
Manages PCI Express NVM Express (NVMe 1.4) solid-state drives and storage namespaces:
```bash
keira> nvme status
NVMe PCIe Storage Controller Subsystem [Active]
  Status      : Ready (Admin SQ/CQ Configured)
  Specification: NVMe 1.4 (PCIe Gen3/Gen4)
  Namespaces  : 1 Active NVMe Namespace (/dev/nvme0n1)
  Capacity    : 1024 MB

keira> nvme identify
NVMe Controller Identification (Identify Controller & Namespaces):
  PCI Address : 0:4.0
  MMIO Base   : 0xFE000000
  NVMe Spec   : 1.4.0 (0x00010400)
  Admin SQ    : 0x01000000 (Depth: 64 entries)
  Admin CQ    : 0x01001000 (Depth: 64 entries)
  Namespaces  : 1 active
```

### `raid`
Monitors and synchronizes Software RAID 0 (striping) and RAID 1 (mirroring) arrays:
```bash
keira> raid status
Software RAID Subsystem [Active]
  Status      : Operational
  RAID Arrays : 2 configured (2 Synced / Optimal)
  Supported   : RAID-0 (Striping), RAID-1 (Mirroring)
  Syscall     : 74 (SYS_RAID_LVM)

keira> raid list
Software RAID State Table:
[RAID] Device /dev/md0 (RAID-0): 2/2 Disks Active - [SYNCED / OK]
[RAID] Device /dev/md1 (RAID-1): 2/2 Disks Active - [SYNCED / OK]
```

### `swap`
Inspects, mounts, or exercises virtual memory disk swap partitions and slot allocation:
```bash
keira> swap on /data/swapfile
Swap Activation: [OK]
  Backing Device : /data/swapfile
  Capacity       : 64 MB (16384 4KB slots)

keira> swap status
Virtual Memory Disk Swap Status: [ACTIVE]
  Backing Device : /data/swapfile
  Total Space    : 64 MB (16384 pages)
  Used Space     : 0 KB (0 pages)
  Free Space     : 64 MB (16384 pages)
  Swap Activity  : 0 In | 0 Out
```

### `kvm`
Inspects hardware virtualization capabilities and exercises isolated guest VM execution:
```bash
keira> kvm status
Kernel-based Virtual Machine (KVM) Subsystem [Active]
  Hardware Ext: Intel VMX=YES, AMD SVM=NO
  Hypervisor  : Active (Bare-Metal KVM Engine)
  Guest VMs   : 1 allocated (Max: 4)
  Total Exits : 0 transitions handled
  Syscalls    : 42 (kvm_create_vm), 43 (kvm_run_vcpu)

keira> kvm list
VM ID   Status    Memory (MB)   vCPUs   Total VM-Exits
  #1    Active    128 MB          1       0

keira> kvm run 1 0
vCPU Execution Transition (VM #1, vCPU #0):
  Exit Reason : CPUID Instruction (Code 3)
  Guest RIP   : 0x0000FFF2
  Guest RSP   : 0x00007C00
  Guest CR0   : 0x60000010
  Total Exits : 1
```

### `lkm`
Manages Loadable Kernel Modules, dynamic symbol tables, and runtime module lifecycles:
```bash
keira> lkm status
Loadable Kernel Module (LKM) Subsystem [Active]
  Status      : Online (Syscall 34 & 35 active)
  Loaded Mods : 3 / 16 active
  Symbol Table: 10 base symbols exported
  Syscalls    : 34 (init_module), 35 (delete_module)

keira> lkm lsmod
Module                  Size  Used by  State     Load Address
ext4_fs                65536        1  Live      0xFFFF800000400000
e1000_nic              32768        0  Live      0xFFFF800000410000
ahci_sata              24576        2  Live      0xFFFF800000420000

keira> lkm symbols
Kernel Symbol Table (kallsyms):
  0xFFFF800000101000        vga_print_str
  0xFFFF800000101200        vga_set_color
  0xFFFF800000102000        klog_write
  0xFFFF800000103000  [GPL] pmm_alloc_frame
  0xFFFF800000103100  [GPL] pmm_free_frame
  0xFFFF800000104000  [GPL] vmm_map_page
  0xFFFF800000105000        scheduler_yield
  0xFFFF800000106000        timer_get_ticks
  0xFFFF800000107000  [GPL] ext4_mount
  0xFFFF800000107200  [GPL] ext4_lookup
```
