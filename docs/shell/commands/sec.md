<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Security & Authentication Shell Commands

This document details all native commands in Keira Kernel related to multi-user authentication, privilege delegation, Seccomp BPF filters, Mandatory Access Control (MAC), and TPM 2.0.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `user` | `user [whoami \| list \| add <name> \| del <name>]` | Query current user account or manage system user identities |
| `login` | `login [username]` | Authenticate and switch user context with password verification |
| `tpm` | `tpm [status \| pcr \| quote \| random]` | Query hardware TPM 2.0 security enclave registers and PCR values |
| `seccomp` | `seccomp [status \| filter \| test]` | Inspect active task Seccomp BPF system call filters |
| `bpf` | `bpf [list \| load <prog> \| status]` | Inspect kernel extended BPF bytecode execution engine |
| `mac` | `mac [status \| enforce \| permissive]` | Query and toggle Mandatory Access Control security policy status |

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

### `tpm status`
Queries the hardware TPM 2.0 controller over memory-mapped I/O:
```bash
keira> tpm status
TPM 2.0 Security Module:
  Manufacturer  : 0x54434720 (TCG)
  Specification : 2.0 (Revision 1.59)
  Status        : Initialized & Armed
  PCR[0] Hash   : e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```
