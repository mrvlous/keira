<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Filesystem & Storage Commands

The `fs` command suite provides low-level block storage device and partition geometry diagnostics.

> [!NOTE]
> **Pure Kernel Demarcation**: Application-level file editing, viewing, copying, directory traversal and removal are handled strictly in Ring 3 userspace via freestanding binaries (`/bin/cat.elf`, `/bin/ls.elf`, `/bin/df.elf`) or the unprivileged POSIX shell (`sh`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `disk` | `disk` | Display block storage devices and partition tables | `-h, --help` |
