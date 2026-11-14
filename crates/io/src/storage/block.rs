// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Generic block device abstraction traits and global registry.

pub mod registry {
    use super::traits::BlockDevice;
    
    const MAX_DEVICES: usize = 4;
    static mut BLOCK_DEVICES: [Option<&'static dyn BlockDevice>; MAX_DEVICES] = [None; MAX_DEVICES];
    static mut MOUNTED_DEVICE: Option<&'static dyn BlockDevice> = None;
    
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
    
    pub fn get_mounted_device() -> Option<&'static dyn BlockDevice> {
        unsafe { MOUNTED_DEVICE }
    }
    
    pub fn flush_mounted_device() -> Result<(), &'static str> {
        if let Some(dev) = get_mounted_device() {
            dev.flush()
        } else {
            Ok(())
        }
    }
    
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
}

pub mod traits {
    pub trait BlockDevice {
        /// Reads a 512-byte sector from the device into the buffer.
        fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str>;
    
        /// Writes a 512-byte sector from the buffer to the device.
        fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str>;
    
        /// Retrieves total capacity of the device in 512-byte sectors.
        fn get_size_sectors(&self) -> u32;
    
        /// Retrieves the human-readable identifier of the device (e.g. "ide0", "ahci0", "ram0").
        fn get_name(&self) -> &'static str;
    
        /// Flushes volatile write caches on the underlying block device hardware.
        fn flush(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

pub use self::registry::*;
pub use self::traits::*;

#[cfg(test)]
mod tests {
    use super::registry::{
        flush_mounted_device, for_each_device, get_device, get_mounted_device, mount_device,
        register_device,
    };
    use super::traits::BlockDevice;
    
    struct DummyDevice {
        name: &'static str,
    }
    
    impl BlockDevice for DummyDevice {
        fn read_sector(&self, _sector: u32, _buffer: &mut [u8; 512]) -> Result<(), &'static str> {
            Ok(())
        }
    
        fn write_sector(&self, _sector: u32, _buffer: &[u8; 512]) -> Result<(), &'static str> {
            Ok(())
        }
    
        fn get_size_sectors(&self) -> u32 {
            100
        }
    
        fn get_name(&self) -> &'static str {
            self.name
        }
    }
    
    static DUMMY_DEV: DummyDevice = DummyDevice { name: "dummy0" };
    
    #[test]
    fn test_block_device_registration_and_mounting() {
        let _ = register_device(&DUMMY_DEV);
        let dev = get_device("dummy0").expect("device must be found");
        assert_eq!(dev.get_name(), "dummy0");
        assert_eq!(dev.get_size_sectors(), 100);
    
        let mounted = mount_device("dummy0");
        assert!(mounted.is_ok());
    
        let cur = get_mounted_device().expect("mounted device present");
        assert_eq!(cur.get_name(), "dummy0");
    
        assert!(flush_mounted_device().is_ok());
    
        let mut found = false;
        for_each_device(|d, is_mounted| {
            if d.get_name() == "dummy0" && is_mounted {
                found = true;
            }
        });
        assert!(found);
    }
}
