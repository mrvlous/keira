// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for block device registration and mounting.

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
