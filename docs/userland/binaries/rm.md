<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `rm.elf` Freestanding Core Utility

The `rm.elf` executable (`/bin/rm.elf` and `/bin/rm`) is a freestanding Ring 3 core utility that removes files and directories from the virtual filesystem.

---

## 1. Architectural Purpose

Filesystem cleanup is handled cleanly from userspace via Syscall 87 (`SYS_UNLINK`):

1. **File Unlinking**: Removes regular file entries and releases backing cluster allocations on disk.
2. **Directory Safety**: Guards against accidental directory unlinking by requiring `-r` (`-R`, `--recursive`).
3. **Recursive Traversal**: Uses `opendir`/`readdir` to walk nested directory structures in userspace and delete leaf entries before removing parent directories.
4. **Force Removal**: Supports `-f` (`--force`) to ignore nonexistent targets without returning an error code.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-f` | `--force` | Ignore nonexistent files and arguments, never prompt |
| `-r`, `-R` | `--recursive` | Remove directories and their contents recursively |
| `-v` | `--verbose` | Explain what is being done during deletion |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Remove a regular file:
keira:/# rm /tmp/sample.txt

# Remove non-existent file with force flag:
keira:/# rm -f /tmp/nonexistent.txt

# Attempt to remove a directory without -r (fails safely):
keira:/# rm /tmp/build
rm: cannot remove '/tmp/build': Is a directory

# Remove a nested directory hierarchy recursively with verbose logging:
keira:/# rm -rv /tmp/build
removed '/tmp/build/output.o'
removed directory '/tmp/build'
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/rm/main.c`
- **Binary Locations**: `/bin/rm.elf`, `/bin/rm`
- **System Call**: Syscall 87 (`SYS_UNLINK`)
- **Privilege Level**: Ring 3 unprivileged
