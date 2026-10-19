<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Privilege & Execution Security

Keira operates strictly as a freestanding Ring 0 monolithic kernel without multi-user distribution bloat (`/etc/passwd`, `/etc/group`, pseudo-PAM, or login managers). All process credentials and privilege boundaries are enforced purely through 32-bit numerical identifiers in Ring 0.

---

## 1. Pure Numerical Credential Model

In accordance with pure kernel design, the kernel credentials engine tracks four integer attributes per task:

- **Real UID (`uid`)**: Numeric user identity of the process owner.
- **Real GID (`gid`)**: Numeric group identity of the process owner.
- **Effective UID (`euid`)**: Numeric identity evaluated during privilege checks and permission resolution.
- **Effective GID (`egid`)**: Numeric group evaluated during group permission resolution.

UID 0 / GID 0 is canonically recognized as `root` (superuser). All other user contexts are represented strictly by their integer values (e.g. UID 1000).

---

## 2. Kernel System Call Interface

Userland processes interact with the credentials engine strictly via canonical POSIX system calls:

| Syscall Vector | Name | Description |
| :--- | :--- | :--- |
| `23` | `SYS_SETUID` | Mutates process UID (restricted to superuser UID 0) |
| `24` | `SYS_GETUID` | Queries real UID of calling process |
| `46` | `SYS_SETGID` | Mutates process GID (restricted to superuser UID 0) |
| `47` | `SYS_GETGID` | Queries real GID of calling process |
| `90` | `SYS_GETEUID` | Queries effective UID of calling process |
| `91` | `SYS_GETEGID` | Queries effective GID of calling process |

---

## 3. Privilege Demotion & Security Boundaries

1. **Privilege Demotion**: Processes running as root (UID 0) can demote themselves to unprivileged numerical IDs (e.g., via `/bin/su 1000`).
2. **Elevation Defense**: Non-root processes (`euid != 0`) attempting to mutate credentials to UID 0 or other IDs are rejected with `EPERM` (`Operation not permitted`).
3. **Task Credential Inheritance**: Child tasks spawned via `execve()` inherit parent process credentials (`parent.uid`, `parent.gid`, `parent.euid`, `parent.egid`), maintaining privilege isolation.

---

## 4. Multi-Layer Security Architecture

* **Kernel Space (Ring 0)**: Unrestricted direct hardware access, interrupt handling and memory page allocation.
* **Userland Sandbox (Ring 3)**: Isolated address spaces per process, preemptive task scheduling and syscall ABI verification.
* **Mandatory Access Control (MAC)**: Capability enforcement and matrix validation restricting process actions at the kernel boundary.
* **Seccomp Filtering**: Dynamic system call whitelisting to sandbox untrusted userland workloads.
