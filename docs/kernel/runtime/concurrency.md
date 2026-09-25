<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Concurrency & Multi-Core Synchronization

Multi-core kernel execution requires strict, race-free synchronization primitives.

---

## Synchronization Primitives

1. **Spinlocks (`SpinMutex`)**: Busy-waiting locks for short critical sections (`crates/core/src/sync/spinlock/`).
2. **IRQ-Safe Spinlocks (`IrqLock`)**: Disables local interrupts while holding the lock to prevent deadlock with ISRs.
3. **Atomic Variables**: Lock-free counters and flags using standard atomic memory orderings (`Acquire`, `Release`, `SeqCst`).
4. **Per-CPU Data (`PerCpu`)**: Isolated per-core contexts eliminating lock contention for core-local metrics.
