// SPDX-License-Identifier: GPL-2.0-only

//! Intel 82540EM (e1000) Gigabit Ethernet controller driver.

pub mod device;
pub mod regs;

pub use device::{
    init, receive_raw_frame, transmit_raw_frame, BYTES_RECEIVED, BYTES_SENT, E1000_FOUND,
    E1000_IO_BASE, E1000_MAC, E1000_MEM_BASE, PACKETS_RECEIVED, PACKETS_SENT,
};
pub use regs::{E1000RxDesc, E1000TxDesc};
