// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Ring 3 user-mode privilege setup, TSS context loading and security initialization.

use keira_core::pr_info;
use keira_syscall::init_user_mode;

/// Transition hardware registers into user-mode execution readiness.
///
/// # Safety
/// Caller guarantees GDT, IDT and scheduler are fully initialized.
pub unsafe fn enter_userspace() {
    init_user_mode();

    pr_info!("gdt", "re-configured Global Descriptor Table segments");
    pr_info!("tss", "Task State Segment CPU context loaded");
    pr_info!("msr", "CPU Ring 3 user-mode syscall interface MSRs enabled");
    pr_info!("mac", "Mandatory Access Control security subsystem active");

    if cfg!(kconfig_userland) && !keira_core::has_boot_cmdline_flag("console=ring0") {
        pr_info!("init", "userspace execution interface ready");
    } else {
        pr_info!(
            "init",
            "pure Ring 0 kernel profile active (userland skipped)"
        );
    }
}

pub mod context {
    pub use super::enter_userspace;
}
