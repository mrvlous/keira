<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Security & Authentication Shell Commands

This document details all native commands in Keira Kernel related to multi-user authentication, privilege delegation, Seccomp BPF filters, Mandatory Access Control (MAC), and TPM 2.0.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `login` | `login [username]` | `[Active]` | Authenticate user credentials against `/config/sys/passwd` and switch session |
| `user` | `user [whoami \| list \| add <name> \| del <name>]` | `[Active]` | Manage system user identities, UID/GID mapping, and home folders in `/users/` |
| `protect` | `protect <path> [readonly \| hidden \| archive]` | `[Active]` | Configure FAT filesystem attributes and access permissions on files |
| `tpm` | `tpm [status \| pcr]` | `[Preview]` | Query Trusted Platform Module (TPM 2.0) hardware security enclave interface |
| `seccomp` | `seccomp [status]` | `[Preview]` | Inspect Secure Computing (Seccomp) syscall filtering sandbox interface (Syscall 36) |
| `bpf` | `bpf [list \| status \| maps]` | `[Preview]` | Inspect Extended Berkeley Packet Filter (eBPF) runtime program interface (Syscall 51 & 52) |
| `mac` | `mac [status \| enforce \| permissive]` | `[Preview]` | Query and toggle Mandatory Access Control (MAC) security policy interface (Syscall 62) |

---

## Detailed Usage

### `user` & `login`
Inspects user credentials and privilege levels:
```bash
keira> user whoami
Current User : admin (UID: 0, GID: 0)
Role         : System Administrator
Home         : /users/admin
Shell        : /system/bin/shell
```

### `protect <path>`
Sets read-only or hidden security attributes on a target FAT16 file:
```bash
keira> protect /config/sys/os-release readonly
[OK] File /config/sys/os-release attribute updated to Read-Only.
```
