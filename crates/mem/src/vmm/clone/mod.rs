// SPDX-License-Identifier: GPL-2.0-only

//! Process address space cloning and PML4 tree duplication.

pub mod address_space;

pub use address_space::{
    clone_kernel_pml4, clone_user_address_space, clone_user_address_space_cow,
};
