// SPDX-License-Identifier: GPL-2.0-only

//! System call argument validation routines.

pub mod fd;
pub mod memory;

pub use fd::validate_fd;
pub use memory::HEAP_MAX_VADDR;
