// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! User space virtual address boundary validation and page fault verification.

pub mod bounds;
pub mod checker;
pub mod fault;

pub use bounds::{check_user_bounds, USER_MAX_ADDR, USER_MIN_ADDR};
pub use checker::validate_user_ptr;
pub use fault::try_fault_user_page;
