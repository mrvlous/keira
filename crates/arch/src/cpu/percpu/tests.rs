// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit test suite for per-CPU memory layout and initialization.

use super::context::{
    get_current_kernel_stack, get_current_percpu, init_percpu, set_current_kernel_stack, PerCpu,
    PER_CPU_DATA,
};

#[test]
fn test_percpu_struct_layout_and_offsets() {
    assert_eq!(core::mem::size_of::<PerCpu>(), 128);
    assert_eq!(core::mem::align_of::<PerCpu>(), 64);

    assert_eq!(core::mem::offset_of!(PerCpu, self_ptr), 0x00);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rsp_scratch), 0x08);
    assert_eq!(core::mem::offset_of!(PerCpu, kernel_stack), 0x10);
    assert_eq!(core::mem::offset_of!(PerCpu, main_stack), 0x18);
    assert_eq!(core::mem::offset_of!(PerCpu, core_id), 0x20);
    assert_eq!(core::mem::offset_of!(PerCpu, syscall_depth), 0x24);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rip), 0x28);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rflags), 0x30);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rbx), 0x38);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rbp), 0x40);
    assert_eq!(core::mem::offset_of!(PerCpu, user_r12), 0x48);
    assert_eq!(core::mem::offset_of!(PerCpu, user_r13), 0x50);
    assert_eq!(core::mem::offset_of!(PerCpu, user_r14), 0x58);
    assert_eq!(core::mem::offset_of!(PerCpu, user_r15), 0x60);
    assert_eq!(core::mem::offset_of!(PerCpu, user_rsp), 0x68);
    assert_eq!(core::mem::offset_of!(PerCpu, current_task_id), 0x70);
}

#[test]
fn test_percpu_initialization_and_stack() {
    unsafe {
        init_percpu(0);
        let percpu = get_current_percpu();
        assert_eq!(percpu.core_id, 0);
        assert_ne!(percpu.kernel_stack, 0);
        assert_eq!(percpu.kernel_stack, percpu.main_stack);
        assert_eq!(percpu.self_ptr, &raw const PER_CPU_DATA[0] as u64);

        set_current_kernel_stack(0x1234_5678_0000);
        assert_eq!(get_current_kernel_stack(), 0x1234_5678_0000);
    }
}
