<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device & Hardware Shell Commands

This document details all native commands in Keira Kernel related to hardware diagnostics, block device control, device drivers, and virtualization.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `devices` | `devices` | List registered character and block device nodes under `/system/dev/` |
| `drivers` | `drivers` | Display loaded device driver descriptors and status (`Active` / `Idle`) |
| `lkm` | `lkm [list \| load <mod> \| unload <mod>]` | Manage loadable kernel modules in the kernel runtime |
| `nvme` | `nvme [info \| list \| stats]` | Display NVMe PCIe controller registers, queues, and namespace stats |
| `usb` | `usb [list \| tree \| ports]` | Enumerate USB host controllers, root hubs, and connected HID/storage devices |
| `kvm` | `kvm [status \| vcpu \| vm]` | Inspect hardware virtualization acceleration (Intel VT-x / AMD-V) |
| `lvm` | `lvm [list \| create <vg> <pv> \| info]` | Display and configure Logical Volume Manager volume groups |
| `raid` | `raid [status \| create <md> <lvl>]` | Inspect software RAID array configuration (RAID 0, RAID 1, RAID 5) |
| `swap` | `swap [status \| on <dev> \| off <dev>]` | Display active swap spaces and configure backing swap devices |
| `epoll` | `epoll [list \| status]` | Inspect kernel `epoll` event descriptors and event subscriptions |

---

## Detailed Usage

### `devices`
Enumerates all device nodes exposed by the Virtual Filesystem layer:
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

### `nvme`
Queries the hardware status of connected NVMe solid-state storage devices over the PCIe bus:
```bash
keira> nvme info
NVMe Controller:
  Vendor ID     : 0x8086
  Device ID     : 0x5845
  Admin Queue   : 64 entries (Ready)
  I/O Queue     : 256 entries (Active)
  Namespaces    : 1 (Total: 1024 MB)
```
