<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `whoami.elf` Freestanding Core Utility

The `whoami.elf` executable (`/bin/whoami.elf` and `/bin/whoami`) is a freestanding Ring 3 core utility that prints the effective user identity of the calling process.

---

## 1. Architectural Purpose

In accordance with Keira's freestanding pure-kernel architecture:

1. **Kernel Numerical Engine**: Queries effective user identity directly from Ring 0 via the canonical `geteuid()` (Vector 90 `SYS_GETEUID`) system call.
2. **Freestanding Representation**: Returns canonical `root` if UID is 0, or prints the numeric user ID (`%u`) for unprivileged identities.
3. **Zero Distro Independence**: Operates with complete zero-stub purity without `/etc/passwd` or foreign distro databases.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage reference and exits with status 0 |
| `-v` | `--version` | Displays version information and exits with status 0 |

---

## 3. Usage Examples

```bash
# Query effective identity as superuser:
keira:/# whoami
root

# Query effective identity under demoted process (e.g. UID 1000):
keira:/$ whoami
1000
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/whoami/main.c`
- **Binary Locations**: `/bin/whoami.elf`, `/bin/whoami`
- **System Call Invocations**: `SYS_GETEUID` (Vector 90)
- **Privilege Level**: Ring 3 unprivileged
