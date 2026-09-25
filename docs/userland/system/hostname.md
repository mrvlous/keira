<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hostname Configuration & Resolution

Manages system network node identification.

---

## Mechanism

* Stored in kernel runtime state and persisted in `/etc/hostname`.
* Configurable at runtime via `sethostname()` system call (`SYS_sethostname`).
* Queried by applications via `gethostname()` system call or ProcFS node (`/proc/sys/kernel/hostname`).
