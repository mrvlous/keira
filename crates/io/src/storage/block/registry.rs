// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Block device registry and mounted filesystem root volume storage.

use super::traits::BlockDevice;

const MAX_DEVICES: usize = 4;
static mut BLOCK_DEVICES: [Option<&'static dyn BlockDevice>; MAX_DEVICES] = [None; MAX_DEVICES];
static mut MOUNTED_DEVICE: Option<&'static dyn BlockDevice> = None;

/// Registers a new block device into the system registry.
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

/// Finds a registered block device by its device identifier string.
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

/// Sets the currently mounted root block storage device by name.
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

/// Retrieves the currently mounted root block device.
pub fn get_mounted_device() -> Option<&'static dyn BlockDevice> {
    unsafe { MOUNTED_DEVICE }
}

/// Flushes volatile write caches on the currently mounted root block device.
pub fn flush_mounted_device() -> Result<(), &'static str> {
    if let Some(dev) = get_mounted_device() {
        dev.flush()
    } else {
        Ok(())
    }
}

/// Iterates through all registered storage block devices and invokes the specified callback.
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
