// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Per-CPU execution state and dedicated kernel stack subsystem.

pub mod context;

#[cfg(test)]
mod tests;

pub use context::{
    get_current_core_id, get_current_kernel_stack, get_current_percpu, get_percpu, init_percpu,
    init_percpu_data, load_percpu_msrs, set_current_kernel_stack, KernelStack, PerCpu,
    PER_CPU_DATA, PER_CPU_STACKS, PER_CPU_STACK_SIZE,
};
