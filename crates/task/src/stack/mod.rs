// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System V ABI initial user stack setup, argument formatting, and ELF auxiliary vector construction.

pub mod auxv;
pub mod user;

#[cfg(test)]
mod tests;

pub use auxv::*;
pub use user::{setup_user_stack_32, setup_user_stack_64};
