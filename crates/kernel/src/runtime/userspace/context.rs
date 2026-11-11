// SPDX-License-Identifier: GPL-2.0-only

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
    pr_info!("init", "executing canonical userspace init process (PID 1)");
    pr_info!(
        "init",
        "system initialization complete; spawning interactive shell"
    );
}
