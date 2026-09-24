// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Intel 82540EM (e1000) Gigabit Ethernet controller driver.

pub mod device;
pub mod regs;

pub use device::{
    init, receive_raw_frame, transmit_raw_frame, E1000_FOUND, E1000_IO_BASE, E1000_MAC,
    E1000_MEM_BASE, PACKETS_RECEIVED, PACKETS_SENT,
};
pub use regs::{E1000RxDesc, E1000TxDesc};
