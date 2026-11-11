// SPDX-License-Identifier: GPL-2.0-only

//! Low-level memory primitives, alignment arithmetic and volatile hardware wrappers.
//!
//! Subdivided into dedicated modules for alignment algorithms, architectural layout
//! constants and volatile MMIO register cells.

pub mod align;
pub mod layout;
pub mod volatile;

pub use align::{align_down, align_up, is_aligned};
pub use layout::{
    BYTE, GIB, KIB, MIB, PAGE_OFFSET_MASK_2M, PAGE_OFFSET_MASK_4K, PAGE_SHIFT_1G, PAGE_SHIFT_2M,
    PAGE_SHIFT_4K, PAGE_SIZE_1G, PAGE_SIZE_2M, PAGE_SIZE_4K,
};
pub use volatile::Volatile;
