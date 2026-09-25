<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Userland System Architecture & Runtime Configuration

Defines kernel privilege boundaries, VFS modes, system initialization, and POSIX I/O models.

---

## Submodule Documents

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`users.md`](users.md) | Privilege Model | Ring 0 vs Ring 3 privileges, MAC boundaries, and sandbox security |
| [`hostname.md`](hostname.md) | Hostname System | Kernel node name storage, `/config/sys/hostname.cfg`, `gethostname`, `sethostname` |
| [`permissions.md`](permissions.md) | File System Modes | VFS node kinds (regular, dir, dev, proc), access characteristics |
| [`init.md`](init.md) | System Initialization | Early userspace initialization, init process (PID 1), and kernel control plane |
| [`posix_io.md`](posix_io.md) | POSIX I/O Architecture | Standard stream descriptors, non-blocking I/O, file offset semantics |
