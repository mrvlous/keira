// SPDX-License-Identifier: GPL-2.0-only

//! Storage block device and filesystem initialization stage.

pub mod mount;

pub use mount::init_storage_and_fs;
