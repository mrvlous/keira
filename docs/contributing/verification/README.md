<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Verification, Testing & Debugging

This directory specifies automated test suites, remote GDB debugging, and unsafe Rust safety contracts in Keira Kernel.

---

## Verification Index

| Document | Topic | Description |
| :--- | :--- | :--- |
| **Testing Suite** | [`testing.md`](testing.md) | Headless smoke testing, QMP scripts, and 20-cycle stress tests |
| **Kernel Debugging** | [`debugging.md`](debugging.md) | Remote GDB debugging on TCP port 1234, register dumps, and QEMU monitor |
| **Unsafe Guidelines** | [`unsafe_guidelines.md`](unsafe_guidelines.md) | Memory safety invariants, user pointer validation, and `# Safety` docs |
