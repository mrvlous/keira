// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interrupt Descriptor Table (IDT) configuration and dispatch subsystem.

pub mod descriptor;
pub mod handlers;

pub use descriptor::{init, load_current_idt, set_gate, IdtEntry, IdtPtr};
pub use handlers::isr_handler;
