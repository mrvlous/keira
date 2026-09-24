// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual Memory Manager (VMM) 4-level paging, VMA address space, and page fault handling.
//!
//! Subdivided into specialized modules for page table entries, hardware traversal,
//! page mapping and protection, virtual memory areas, demand paging, cloning, and reclamation.

pub mod area;
pub mod clone;
pub mod fault;
pub mod mapping;
pub mod reclaim;
pub mod table;

pub use area::*;
pub use clone::*;
pub use fault::*;
pub use mapping::*;
pub use reclaim::*;
pub use table::*;

#[cfg(test)]
mod tests;
