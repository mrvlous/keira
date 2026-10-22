<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `umount.elf` Freestanding Core Utility

The `umount.elf` executable (`/bin/umount.elf` and `/bin/umount`) is a freestanding Ring 3 core utility that detaches filesystems from the Virtual File System (VFS) hierarchy.

---

## 1. Architectural Purpose

Filesystem detachment is performed safely through the kernel system call gateway:

1. **Namespace Cleanliness**: Unmounts mounted directory endpoints and reclaims associated volatile memory resources.
2. **Rootfs Guard**: Prevents unmounting the root filesystem (`/`), returning `EBUSY` when attempted.
3. **Privilege Enforcement**: Enforces root UID verification before privileged detachment operations are dispatched via `SYS_UMOUNT` (vector 95).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-f` | `--force` | Forces unmounting of target filesystem |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Detach an active mount point:
keira:/# umount /mnt
umount: successfully unmounted /mnt

# Verify unmounted state:
keira:/# mount
/dev/ahci0 on / type fat (rw,relatime)
procfs on /proc type procfs (ro,nosuid,nodev,noexec,relatime)
devfs on /dev type devfs (rw,nosuid,relatime)
initrd on /initrd type initrd (ro,relatime)
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/umount/main.c`
- **Binary Locations**: `/bin/umount.elf`, `/bin/umount`
- **Privilege Level**: Ring 3, root UID required
- **Underlying Syscall**: `SYS_UMOUNT` (95)
