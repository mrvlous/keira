// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Generic block storage device trait abstraction.

/// Trait defining operations for 512-byte sector-addressable block storage hardware.
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
