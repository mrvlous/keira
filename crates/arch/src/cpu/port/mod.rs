// SPDX-License-Identifier: GPL-2.0-only

//! x86/x86_64 Port-Mapped I/O (PMIO) subsystem.

pub mod io;

pub use io::{inb, inl, inw, io_wait, outb, outl, outw};
