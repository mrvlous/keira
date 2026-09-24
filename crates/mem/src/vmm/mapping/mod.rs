// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page mapping, unmapping, protection attributes, and TLB invalidation.

pub mod map;
pub mod protect;
pub mod unmap;

pub use map::{map_huge_2m_page, map_page, map_page_in_pml4};
pub use protect::{is_user_page_mapped, mprotect_page};
pub use unmap::{free_and_unmap_page, unmap_huge_2m_page, unmap_page};
