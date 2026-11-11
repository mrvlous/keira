// SPDX-License-Identifier: GPL-2.0-only

//! VirtIO-Net network interface driver module.

pub mod device;

pub use device::{VirtioNetDevice, VIRTIO_NET_DEVICE_ID, VIRTIO_NET_VENDOR_ID};
