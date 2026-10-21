<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `ln.elf` Freestanding Core Utility

The `ln.elf` executable (`/bin/ln.elf` and `/bin/ln`) is a freestanding Ring 3 core utility that creates symbolic links to files and directories across Keira's monolithic Virtual File System (VFS).

---

## 1. Architectural Purpose

In accordance with Keira's freestanding pure-kernel architecture:

1. **Kernel Symlink Engine**: Interacts directly with Ring 0 via the canonical `symlink()` (Vector 92 `SYS_SYMLINK`) and `readlink()` (Vector 93 `SYS_READLINK`) system calls.
2. **Transparent VFS Traversal**: Created links are registered into the VFS in-kernel symbolic link registry, allowing standard operations (`open`, `read`, `write`, `stat` and `load_elf`) to transparently dereference target paths up to a recursion depth of 8.
3. **Atomic Unlink Separation**: Removing a symbolic link via `/bin/rm` or `unlink()` unbinds the link itself without destroying or modifying the underlying target file.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-s` | `--symbolic` | Creates a symbolic link pointing to the specified target path |
| `-f` | `--force` | Removes existing destination files before establishing the link |
| `-v` | `--verbose` | Prints link creation confirmations (`'link' -> 'target'`) |
| `-h` | `--help` | Displays usage reference and exits with status 0 |

---

## 3. Usage Examples

```bash
# Create a symbolic link pointing to kernel configuration:
keira:/# ln -s /etc/kernel.cfg /tmp/cfg_link

# Verify transparent target resolution:
keira:/# cat /tmp/cfg_link
KERNEL_NAME=keira
KERNEL_VERSION=0.7.0
KERNEL_ARCH=x86_64

# Force overwrite of an existing destination link with verbose output:
keira:/# ln -sfv /bin/ls /tmp/my_ls
'/tmp/my_ls' -> '/bin/ls'

# Execute the target binary via the symbolic link:
keira:/# /tmp/my_ls /etc
kernel.cfg  hostname

# Unlink the link without affecting the target:
keira:/# rm /tmp/cfg_link
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/ln/main.c`
- **Binary Locations**: `/bin/ln.elf`, `/bin/ln`
- **System Call Invocations**: `SYS_SYMLINK` (Vector 92) and `SYS_READLINK` (Vector 93)
- **Privilege Level**: Ring 3 unprivileged
