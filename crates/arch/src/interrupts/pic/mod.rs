// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dual 8259 Programmable Interrupt Controller (PIC) subsystem.

pub mod i8259;

pub use i8259::{
    clear_mask, init, pic_clear_mask, pic_eoi, pic_set_mask, send_eoi, set_mask, PIC1_COMMAND,
    PIC1_DATA, PIC2_COMMAND, PIC2_DATA,
};
