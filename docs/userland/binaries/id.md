<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `id.elf` Freestanding Core Utility

The `id.elf` executable (`/bin/id.elf` and `/bin/id`) is a freestanding Ring 3 core utility that prints real and effective user ID (UID) and group ID (GID) credentials from the kernel credentials subsystem.

---

## 1. Architectural Purpose

In accordance with Keira's freestanding pure-kernel architecture:

1. **Kernel Credential Introspection**: Obtains real and effective user and group identities directly from the kernel via `getuid()` (Vector 24), `geteuid()` (Vector 90), `getgid()` (Vector 47), and `getegid()` (Vector 91).
2. **Standard POSIX Output**: Emits the standard `uid=... gid=... groups=...` output format, identifying UID 0 / GID 0 as `root` and unprivileged identities as numeric IDs.
3. **Explicit Identity Queries**: Allows querying arbitrary numerical identities (e.g. `id 1000` or `id root`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-u` | `--user` | Print only the effective user ID |
| `-g` | `--group` | Print only the effective group ID |
| `-G` | `--groups` | Print all group IDs |
| `-r` | `--real` | Print real ID instead of effective ID (with `-u`, `-g`, `-G`) |
| `-h` | `--help` | Displays usage reference and exits with status 0 |
| `-v` | `--version` | Displays version information and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display full credentials as root:
keira:/# id
uid=0(root) gid=0(root) groups=0(root)

# Query user ID only:
keira:/# id -u
0

# Query numeric identity details:
keira:/# id 1000
uid=1000 gid=1000 groups=1000

# Display credentials under demoted process:
keira:/$ id
uid=1000 gid=1000 groups=1000
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/id/main.c`
- **Binary Locations**: `/bin/id.elf`, `/bin/id`
- **System Call Invocations**: `SYS_GETUID` (Vector 24), `SYS_GETEUID` (Vector 90), `SYS_GETGID` (Vector 47), `SYS_GETEGID` (Vector 91)
- **Privilege Level**: Ring 3 unprivileged
