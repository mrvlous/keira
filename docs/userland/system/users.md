<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Privilege & Execution Security

Keira operates strictly as a freestanding Ring 0 monolithic kernel without multi-user distribution bloat (`/etc/passwd`, `/etc/group`, pseudo-PAM, or login managers).

---

## Security Model

* **Kernel Space (Ring 0)**: Unrestricted direct hardware access, interrupt handling, and memory page allocation.
* **Userland Sandbox (Ring 3)**: Isolated address spaces per process, preemptive task scheduling, and syscall ABI verification.
* **Mandatory Access Control (MAC)**: Capability enforcement and matrix validation restricting process actions at the kernel boundary.
* **Seccomp Filtering**: Dynamic system call whitelisting to sandbox untrusted userland workloads.
