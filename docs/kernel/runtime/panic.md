<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Panic & Crash Telemetry

When an unrecoverable kernel-level invariant fails, Keira executes the centralized panic handler (`crates/kernel/src/diagnostic/`).

---

## Panic Sequence

1. **Disable Interrupts**: `CLI` executed across all CPU cores.
2. **VGA Visual Alert**: Screen switched to dark blue background with white/red text.
3. **Diagnostic Dumps**: Prints panic message, source code line number, and CPU register snapshots (`RAX`, `RBX`, `RIP`, `RSP`, `CR2`, `CR3`).
4. **Halt**: Places the CPU into a permanent halted state.
