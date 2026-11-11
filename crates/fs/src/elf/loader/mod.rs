// SPDX-License-Identifier: GPL-2.0-only

//! Hardened ELF binary loader and user mode dispatcher.

pub mod buffer;
pub mod exec;
pub mod load;
pub mod mapping;
pub mod shebang;

pub use buffer::{last_loaded_elf_slice, ELF_FILE_BUF, LAST_LOADED_LEN};
pub use exec::execute_user_mode;
pub use load::load_elf;
pub use mapping::{handle_load_failure, rollback_all_segments, SegmentMapping, MAX_LOAD_SEGMENTS};
pub use shebang::parse_shebang;
