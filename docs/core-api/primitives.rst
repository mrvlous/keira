.. SPDX-License-Identifier: GPL-2.0-only


Type-Safe Epoch-Based Reclamation (EBR)
=======================================

Epoch-Based Reclamation (EBR) provides lock-free, zero-contention memory reclamation across multicore SMP CPUs with compile-time safety guarantees (``crates/core/src/sync/ebr.rs``).

1. Architectural Motivation
===========================

Traditional reader-writer spinlocks introduce cacheline bouncing and lock contention across CPU cores during read-heavy kernel transactions. While Linux implements RCU (Read-Copy-Update), its implementation in C lacks compile-time lifecycle enforcement and relies on manual grace-period tracking.

Keira's EBR subsystem guarantees:

* **Zero Contention**: Reading threads do not acquire shared mutexes or spinlocks.
* **Compile-Time Safety**: An RAII ``EpochGuard`` automatically unpins the participant upon dropping, preventing stalled grace periods.
* **Deterministic Reclamation**: Memory objects retired in epoch :math:`e` are safely deallocated once the global epoch reaches at least :math:`e + 2`.

2. API Reference
================

.. list-table::
   :header-rows: 1
   :widths: 23 90 86

   * - Primitive
     - Signature
     - Description
   * - ``pin()``
     - ``fn pin() -> EpochGuard``
     - Pins the current CPU core to the active global epoch and returns an RAII guard.
   * - ``current_epoch()``
     - ``fn current_epoch() -> usize``
     - Reads the current monotonically advancing global epoch counter.
   * - ``try_advance_epoch()``
     - ``fn try_advance_epoch() -> bool``
     - Advances the global epoch if all pinned readers have caught up with the current epoch.
   * - ``GarbageBag::retire()``
     - ``fn retire(&mut self, ptr: usize, reclaim: Option<fn(usize)>) -> Result<(), &'static str>``
     - Enqueues a retired pointer or frame under the current global epoch.
   * - ``GarbageBag::collect()``
     - ``fn collect(&mut self) -> usize``
     - Frees all retired resources whose epoch is older than ``current_epoch() - 1``.

3. Kernel Version Metadata API
==============================

The core crate (``crates/core/src/lib.rs``) exposes canonical kernel release version and SemVer components extracted directly at compile time from the root ``Cargo.toml`` workspace package:

.. list-table::
   :header-rows: 1
   :widths: 28 25 47

   * - Constant
     - Type
     - Description
   * - ``KERNEL_VERSION``
     - ``&'static str``
     - Complete SemVer string (e.g. ``"0.8.0"``) from ``env!("CARGO_PKG_VERSION")``.
   * - ``KERNEL_VERSION_MAJOR``
     - ``&'static str``
     - Major version number from ``env!("CARGO_PKG_VERSION_MAJOR")``.
   * - ``KERNEL_VERSION_MINOR``
     - ``&'static str``
     - Minor version number from ``env!("CARGO_PKG_VERSION_MINOR")``.
   * - ``KERNEL_VERSION_PATCH``
     - ``&'static str``
     - Patch version number from ``env!("CARGO_PKG_VERSION_PATCH")``.
