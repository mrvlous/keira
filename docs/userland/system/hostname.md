<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hostname Configuration & Resolution

Manages system network node identification.

---

## Mechanism

* Stored in kernel runtime state and persisted in `/config/sys/hostname.cfg` on primary storage.
* Configurable at runtime via `sethostname()` system call (`SYS_sethostname`) or shell `hostname` command.
* Queried by applications via `gethostname()` system call or ProcFS node (`/proc/sys/kernel/hostname`).
