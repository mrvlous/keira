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

//! Core kernel subsystem initialization pipeline.

pub mod mem;
pub mod net;
pub mod security;
pub mod smp;
pub mod storage;

#[cfg(test)]
mod tests;

pub use mem::init_memory;
pub use net::init_network;
pub use security::init_security_and_measure;
pub use smp::init_smp_and_timers;
pub use storage::init_storage_and_fs;
