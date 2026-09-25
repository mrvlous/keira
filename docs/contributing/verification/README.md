<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Verification, Testing & Debugging Manuals

This directory documents verification procedures, automated test harnesses, QEMU debugging configurations, and safety invariant validation in Keira Kernel.

---

## Verification Index

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`testing.md`](testing.md) | Automated Testing | QEMU smoke tests, `test_abi`, `fuzz_abi`, and CI verification |
| [`debugging.md`](debugging.md) | GDB & Diagnostics | QEMU GDB stub debugging, serial logging, and panic unwinding |
| [`unsafe_guidelines.md`](unsafe_guidelines.md) | Unsafe Rust Safety | Preconditions, invariants, and `# Safety` documentation rules |
