// SPDX-License-Identifier: GPL-2.0-only

//! Page Fault (#PF) interrupt dispatching, stack growth and COW fault resolution.

pub mod handler;

pub use handler::{
    handle_page_fault, vmm_get_cow_stats, vmm_get_fault_stats, COW_IN_PLACE_PROMOTIONS,
    COW_REPLICATIONS, USER_STACK_BOTTOM, USER_STACK_TOP,
};
