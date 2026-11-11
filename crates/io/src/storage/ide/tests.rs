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

//! Unit tests for IDE device properties and LBA bounds.

use super::device::IdeBlockDevice;
use super::driver::read_sector;
use crate::storage::block::BlockDevice;

#[test]
fn test_ide_device_metadata() {
    let dev = IdeBlockDevice { size_sectors: 4096 };
    assert_eq!(dev.get_name(), "ide0");
    assert_eq!(dev.get_size_sectors(), 4096);
}

#[test]
fn test_ide_lba28_out_of_bounds() {
    let mut buf = [0u8; 512];
    unsafe {
        let res = read_sector(0x1000_0000, &mut buf);
        assert!(res.is_err());
    }
}
