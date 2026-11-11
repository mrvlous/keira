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

//! Unit tests for RAM disk block device bounds and initialization.

use super::device::RamBlockDevice;
use crate::storage::block::BlockDevice;

#[test]
fn test_ramdisk_device_properties() {
    let dev = RamBlockDevice::new();
    assert_eq!(dev.get_name(), "ram0");
    assert_eq!(dev.get_size_sectors(), 0);
    assert_eq!(dev.block_size(), 512);
    assert_eq!(dev.block_count(), 0);

    let mut buf = [0u8; 512];
    assert!(dev.read_sector(0, &mut buf).is_err());
    assert!(dev.write_sector(0, &buf).is_err());
}
