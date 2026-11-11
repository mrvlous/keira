// SPDX-License-Identifier: GPL-2.0-only

//! Page table reclamation and user address space teardown.

pub mod tree;

pub use tree::free_user_pages;
