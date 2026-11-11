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

//! Page Fault (#PF) interrupt dispatching, stack growth and COW fault resolution.

pub mod handler;

pub use handler::{
    handle_page_fault, vmm_get_cow_stats, vmm_get_fault_stats, COW_IN_PLACE_PROMOTIONS,
    COW_REPLICATIONS, USER_STACK_BOTTOM, USER_STACK_TOP,
};
