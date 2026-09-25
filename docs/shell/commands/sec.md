<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Security & Sandboxing Commands

The `sec` command suite provides system call sandboxing, mandatory access control, and in-kernel virtual machine bytecode verification.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `bpf` | `bpf` | Inspect BPF bytecode program table, maps, and verifier state | `-h, --help` |
| `mac` | `mac [enforce\|permissive]` | Query or toggle Mandatory Access Control (MAC) domain security | `-h, --help` |
| `seccomp` | `seccomp` | Inspect active seccomp syscall filter profiles and restrictions | `-h, --help` |
