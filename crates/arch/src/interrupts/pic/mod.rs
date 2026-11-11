// SPDX-License-Identifier: GPL-2.0-only

//! Dual 8259 Programmable Interrupt Controller (PIC) subsystem.

pub mod i8259;

pub use i8259::{
    clear_mask, init, pic_clear_mask, pic_eoi, pic_set_mask, send_eoi, set_mask, PIC1_COMMAND,
    PIC1_DATA, PIC2_COMMAND, PIC2_DATA,
};
