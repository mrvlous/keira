// SPDX-License-Identifier: GPL-2.0-only

//! FAT on-disk structures, directory entries, VFAT LFN descriptors and volume layout.

pub mod entry;
pub mod lfn;
pub mod volume;

pub use self::entry::DirectoryEntry;
pub use self::lfn::{LfnAccumulator, LfnEntry};
pub use self::volume::{Fat16Volume, FoundEntry};
