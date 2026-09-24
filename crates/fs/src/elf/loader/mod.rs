// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardened ELF binary loader and user mode dispatcher.

pub mod buffer;
pub mod exec;
pub mod load;
pub mod mapping;

pub use buffer::{last_loaded_elf_slice, ELF_FILE_BUF, LAST_LOADED_LEN};
pub use exec::execute_user_mode;
pub use load::load_elf;
pub use mapping::{handle_load_failure, rollback_all_segments, SegmentMapping, MAX_LOAD_SEGMENTS};
