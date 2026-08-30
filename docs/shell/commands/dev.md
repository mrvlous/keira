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
| `epoll` | `epoll [list \| status]` | `[Preview]` | Inspect kernel `epoll` event multiplexer interface (Syscall 55 & 56) |
| `kvm` | `kvm [status \| vcpu \| vm]` | `[Preview]` | Inspect hardware virtualization acceleration interface (Syscall 49 & 50) |
| `lkm` | `lkm [list \| load <mod> \| unload <mod>]` | `[Preview]` | Inspect Loadable Kernel Module symbols and dynamic resolution (Syscall 34 & 35) |
| `lvm` | `lvm [list \| create <vg> <pv> \| info]` | `[Preview]` | Display and configure Logical Volume Manager volume groups |
| `nvme` | `nvme [info \| list \| stats]` | `[Preview]` | Display NVMe PCIe controller registers, queues, and namespace stats |
| `raid` | `raid [status \| create <md> <lvl>]` | `[Preview]` | Inspect software RAID array configuration (RAID 0, RAID 1) |
| `swap` | `swap [status \| on <dev> \| off <dev>]` | `[Preview]` | Display active swap spaces and configure backing swap devices (Syscall 53 & 54) |

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
