<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `mkdir.elf` Freestanding Core Utility

The `mkdir.elf` executable (`/bin/mkdir.elf` and `/bin/mkdir`) is a freestanding Ring 3 core utility that creates directories in the virtual filesystem.

---

## 1. Architectural Purpose

Directory node instantiation is delegated cleanly to userspace via Syscall 88 (`SYS_MKDIR`):

1. **Hierarchical Path Creation**: Supports creating nested directories in a single command with `-p` (`--parents`).
2. **Standard POSIX Compliance**: Implements standard error semantics, rejecting existing paths unless `-p` is specified and returning standard exit status codes.
3. **KFS Self-Hosting Foundation**: Required by build systems, package extractors and shell scripts to create build directories without external assistance.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-p` | `--parents` | No error if existing, create intermediate parent directories as needed |
| `-v` | `--verbose` | Print a diagnostic confirmation message for each directory created |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Create a single directory:
keira:/# mkdir /tmp/build

# Create nested directory hierarchy with verbose logging:
keira:/# mkdir -pv /tmp/src/keira/crates
mkdir: created directory '/tmp/src'
mkdir: created directory '/tmp/src/keira'
mkdir: created directory '/tmp/src/keira/crates'

# Inspect created directory:
keira:/# ls -a /tmp/src/keira
.  ..  crates
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/mkdir/main.c`
- **Binary Locations**: `/bin/mkdir.elf`, `/bin/mkdir`
- **System Call**: Syscall 88 (`SYS_MKDIR`)
- **Privilege Level**: Ring 3 unprivileged
