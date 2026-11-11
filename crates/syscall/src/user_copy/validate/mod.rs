// SPDX-License-Identifier: GPL-2.0-only

//! User space virtual address boundary validation and page fault verification.

pub mod bounds;
pub mod checker;
pub mod fault;

pub use bounds::{check_user_bounds, USER_MAX_ADDR, USER_MIN_ADDR};
pub use checker::validate_user_ptr;
pub use fault::try_fault_user_page;
