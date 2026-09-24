// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Low-level atomic memory barriers, fences, and processor hints.
//!
//! Provides hardware and compiler synchronization barriers enforcing strict
//! memory ordering between instruction pipelines and device DMA controllers.

use core::sync::atomic::{compiler_fence, fence, Ordering};

/// Emits an instruction to pause the processor pipeline in spin-wait loops.
///
/// Reduces power consumption and eliminates pipeline stall penalties on hyper-threaded
/// CPU cores when repeatedly polling atomic lock status.
#[inline(always)]
pub fn spin_loop_hint() {
    core::hint::spin_loop();
}

/// Enforces a compiler barrier preventing compiler reordering of memory accesses.
///
/// Does not emit hardware fence instructions; purely restricts compiler code generation.
#[inline(always)]
pub fn memory_barrier_compiler(order: Ordering) {
    compiler_fence(order);
}

/// Enforces a hardware memory fence across processor cores.
///
/// Ensures memory operations initiated prior to the fence become globally visible
/// to other cores and DMA controllers before subsequent operations proceed.
#[inline(always)]
pub fn memory_barrier_hardware(order: Ordering) {
    fence(order);
}
