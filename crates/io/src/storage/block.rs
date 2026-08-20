// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Generic block device trait abstraction and system block device registry.

/// Trait defining operations for 512-byte sector-addressable block storage hardware.
pub trait BlockDevice {
    /// Read a 512-byte sector from the device into the buffer.
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str>;

    /// Write a 512-byte sector from the buffer to the device.
    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str>;

    /// Get total capacity of the device in 512-byte sectors.
    fn get_size_sectors(&self) -> u32;

    /// Get the human-readable identifier of the device (e.g. "ide0", "ahci0", "ram0").
    fn get_name(&self) -> &'static str;
}

const MAX_DEVICES: usize = 4;
static mut BLOCK_DEVICES: [Option<&'static dyn BlockDevice>; MAX_DEVICES] = [None; MAX_DEVICES];
static mut MOUNTED_DEVICE: Option<&'static dyn BlockDevice> = None;

/// Register a new block device into the system registry.
pub fn register_device(dev: &'static dyn BlockDevice) -> Result<(), &'static str> {
    unsafe {
        for slot in (&mut *core::ptr::addr_of_mut!(BLOCK_DEVICES)).iter_mut() {
            if slot.is_none() {
                *slot = Some(dev);
                return Ok(());
            }
        }
    }
    Err("Block device registry is full")
}

/// Find a registered block device by its device identifier string.
pub fn get_device(name: &str) -> Option<&'static dyn BlockDevice> {
    unsafe {
        for dev in (&*core::ptr::addr_of!(BLOCK_DEVICES)).iter().flatten() {
            if dev.get_name() == name {
                return Some(*dev);
            }
        }
    }
    None
}

/// Set the currently mounted root block storage device.
pub fn mount_device(name: &str) -> Result<&'static dyn BlockDevice, &'static str> {
    if let Some(dev) = get_device(name) {
        unsafe {
            MOUNTED_DEVICE = Some(dev);
        }
        Ok(dev)
    } else {
        Err("Device not found")
    }
}

/// Get the currently mounted root block device.
pub fn get_mounted_device() -> Option<&'static dyn BlockDevice> {
    unsafe { MOUNTED_DEVICE }
}

/// Iterate through all registered storage block devices and invoke callback.
pub fn for_each_device<F>(mut f: F)
where
    F: FnMut(&'static dyn BlockDevice, bool),
{
    unsafe {
        let mounted_name = MOUNTED_DEVICE.map(|d| d.get_name()).unwrap_or("");
        for dev in (&*core::ptr::addr_of!(BLOCK_DEVICES)).iter().flatten() {
            let is_mounted = dev.get_name() == mounted_name;
            f(*dev, is_mounted);
        }
    }
}
