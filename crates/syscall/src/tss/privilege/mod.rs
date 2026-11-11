// SPDX-License-Identifier: GPL-2.0-only

//! User mode initialization, descriptor configuration and MSR setup.

pub mod gdt;
pub mod init;
pub mod msr;

pub use gdt::configure_gdt_tss;
pub use init::init_user_mode;
pub use msr::configure_syscall_msrs;
