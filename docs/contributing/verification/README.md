<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Testing, Verification & Debugging

This submodule specifies automated testing harnesses, headless QEMU smoke tests, remote GDB debugging, serial logs, and unsafe Rust safety contracts in Keira Kernel.

---

## Verification & Debugging Index

| Component | Document | Description |
| :--- | :--- | :--- |
| **Testing Harness** | [`testing.md`](testing.md) | Unit tests, headless QEMU smoke testing, and multi-architecture verification |
| **Remote Debugging** | [`debugging.md`](debugging.md) | Remote GDB server (`localhost:1234`), COM1 serial logging, and QEMU monitor |
| **Unsafe Safety Contracts** | [`unsafe_guidelines.md`](unsafe_guidelines.md) | Soundness rules, raw pointer dereferencing, and `# Safety` docstrings |
