// SPDX-License-Identifier: GPL-2.0-only

//! Character device node implementations (`null`, `zero`, `random`, `tty`, `ptmx`).

pub mod dispatcher;
pub mod null;
pub mod random;
pub mod tty;
pub mod zero;

pub use self::dispatcher::{exists, get_dev_entry, read_dev_node, write_dev_node, DEV_ENTRIES};
