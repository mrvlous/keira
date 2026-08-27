<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Userland System Services & Security

This submodule specifies multi-user authentication, file permission models, network hostname, init process, and POSIX I/O in Keira Kernel.

---

## System Services Index

| Component | Document | Description |
| :--- | :--- | :--- |
| **Authentication** | [`users.md`](users.md) | Multi-user database (`/config/sys/passwd`), SHA-256 password hashing |
| **File Security** | [`permissions.md`](permissions.md) | POSIX mode bits, read-only file attributes, and access control validation |
| **Network Identity** | [`hostname.md`](hostname.md) | System hostname configuration and persistence (`/config/sys/hostname.cfg`) |
| **Init Process** | [`init.md`](init.md) | Userland PID 1 init process lifecycle and startup script execution |
| **POSIX I/O** | [`posix_io.md`](posix_io.md) | File descriptor table (0..15), standard streams, and `fcntl` access flags |
