<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `df.elf` Freestanding Core Utility

The `df.elf` executable (`/bin/df.elf` and `/bin/df`) is a freestanding Ring 3 core utility that reports filesystem disk space usage.

---

## 1. Architectural Purpose

Filesystem telemetry is accessed from userspace without embedding complex formatting logic inside Ring 0:

1. **Mount Table Inspection**: Queries active filesystem mounts (`/`, `/proc`, `/sys`, `/dev`, `/tmp`).
2. **Space Utilization Reporting**: Displays 1K-blocks capacity, used blocks, available blocks, capacity percentage and mount points.
3. **Purity**: Enables POSIX scripts to verify disk availability prior to package installation or data logging.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display disk free usage table:
keira:/# df
Filesystem     Type    1K-blocks      Used Available Use% Mounted on
/dev/sda1      fat16       32768      4128     28640  13% /
procfs         proc            0         0         0   0% /proc
sysfs          sys             0         0         0   0% /sys
devfs          dev             0         0         0   0% /dev
tmpfs          tmp          4096       128      3968   3% /tmp
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/df/main.c`
- **Binary Locations**: `/bin/df.elf`, `/bin/df`
- **Privilege Level**: Ring 3 unprivileged
