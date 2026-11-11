// SPDX-License-Identifier: GPL-2.0-only

//! Realtek RTL8139 Fast Ethernet driver module.

pub mod device;

pub use device::{Rtl8139Device, RTL8139_DEVICE_ID, RTL8139_VENDOR_ID};
