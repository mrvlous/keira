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

//! Executable and Linkable Format (ELF) header and program header types.

pub mod header;
pub mod program;

pub use header::{Elf32Header, ElfHeader};
pub use program::{
    Program32Header, ProgramHeader, PF_R, PF_W, PF_X, PT_LOAD, USER_MAX_VADDR, USER_MIN_VADDR,
};
