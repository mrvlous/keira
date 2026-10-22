<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `mount.elf` Freestanding Core Utility

The `mount.elf` executable (`/bin/mount.elf` and `/bin/mount`) is a freestanding Ring 3 core utility that attaches filesystems to the Virtual File System (VFS) hierarchy and displays active mount points.

---

## 1. Architectural Purpose

Filesystem mounting and namespace inspection are delegated to autonomous userland:

1. **Mount Table Inspection**: Invocations without arguments query active filesystem mounts formatted dynamically from `/proc/mounts`.
2. **Filesystem Attachment**: Attaches storage devices or virtual filesystems (`tmpfs`, `fat`, `ext4`) to target directory mount points via `SYS_MOUNT` (vector 94).
3. **Privilege Enforcement**: Enforces root UID verification before privileged mounting operations are dispatched.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-t <type>` | `--types` | Specifies the filesystem type (`fat`, `tmpfs`, `ext4`, `procfs`, `devfs`) |
| `-r` | `--read-only` | Mounts the target filesystem with read-only flag (`MS_RDONLY`) |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display all currently mounted filesystems:
keira:/# mount
/dev/ahci0 on / type fat (rw,relatime)
procfs on /proc type procfs (ro,nosuid,nodev,noexec,relatime)
devfs on /dev type devfs (rw,nosuid,relatime)
initrd on /initrd type initrd (ro,relatime)

# Mount an in-memory volatile scratch filesystem at /mnt:
keira:/# mount -t tmpfs none /mnt
mount: successfully mounted none on /mnt (type tmpfs)

# Verify updated mount points:
keira:/# mount
/dev/ahci0 on / type fat (rw,relatime)
procfs on /proc type procfs (ro,nosuid,nodev,noexec,relatime)
devfs on /dev type devfs (rw,nosuid,relatime)
initrd on /initrd type initrd (ro,relatime)
none on /mnt type tmpfs (rw,relatime)
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/mount/main.c`
- **Binary Locations**: `/bin/mount.elf`, `/bin/mount`
- **Privilege Level**: Ring 3 unprivileged (inspection), root UID required (mounting)
- **Underlying Syscall**: `SYS_MOUNT` (94)
