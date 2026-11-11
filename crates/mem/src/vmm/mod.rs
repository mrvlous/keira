// SPDX-License-Identifier: GPL-2.0-only

//! Virtual Memory Manager (VMM) 4-level paging, VMA address space and page fault handling.
//!
//! Subdivided into specialized modules for page table entries, hardware traversal,
//! page mapping and protection, virtual memory areas, demand paging, cloning and reclamation.

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
