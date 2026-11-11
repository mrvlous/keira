// SPDX-License-Identifier: GPL-2.0-only

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
