<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Userland System Architecture & Runtime Configuration

Defines multi-user credentials, permissions, system initialization, and POSIX I/O models.

---

## Submodule Documents

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`users.md`](users.md) | Multi-User Model | User accounts, `/etc/passwd`, `/etc/group`, UIDs, GIDs, root privilege |
| [`hostname.md`](hostname.md) | Hostname System | System hostname storage, `/etc/hostname`, `gethostname`, `sethostname` |
| [`permissions.md`](permissions.md) | File Permissions | POSIX permission bits (`rwxrwxrwx`), umask, ownership validation |
| [`init.md`](init.md) | System Initialization | Early userspace initialization, init process (PID 1), daemon startup |
| [`posix_io.md`](posix_io.md) | POSIX I/O Architecture | Standard stream descriptors, non-blocking I/O, file offset semantics |
