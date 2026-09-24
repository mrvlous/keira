// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Storage block device discovery, FAT16 filesystem mounting, and VMM demand paging.

use keira_fs::fat;
use keira_io::bus::pci;
use keira_io::storage::{ahci, block, ide};
use keira_io::vga;

fn file_backing_read(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    keira_fs::vfs::read_file_offset(path, offset, buf)
}

fn file_backing_sync(path: &str, offset: u64, buf: &[u8]) -> Result<usize, &'static str> {
    let res = keira_fs::vfs::write_file_offset(path, offset, buf)?;
    unsafe {
        let _ = keira_fs::flush_dirty_sectors();
    }
    Ok(res)
}

/// Initialize PCI host controllers, storage devices, root FAT16 filesystem, and VMM hooks.
///
/// # Safety
/// Caller guarantees virtual memory and scheduler primitives are active.
pub unsafe fn init_storage_and_fs() {
    vga::print_boot_log("Initializing PCI Bus & storage/network host controllers", 0);
    pci::init();
    let _ = ahci::init();

    let mut mounted = false;
    if block::mount_device("ahci0").is_ok() {
        mounted = true;
    } else if let Ok(sectors) = ide::identify() {
        ide::IDE_DEVICE.size_sectors = sectors;
        let _ = block::register_device(&*core::ptr::addr_of!(ide::IDE_DEVICE));
        if block::mount_device("ide0").is_ok() {
            mounted = true;
        }
    }

    match fat::init() {
        Ok(_) => {
            if mounted {
                if let Some(dev) = block::get_mounted_device() {
                    if dev.get_name() == "ahci0" {
                        vga::print_boot_log("Probing SATA master storage controller via AHCI", 0);
                    } else {
                        vga::print_boot_log("Probing IDE primary master storage controller", 0);
                    }
                }
            } else {
                vga::print_boot_log("Probing primary storage controller", 0);
            }
            vga::print_boot_log("Registering active storage block device drives", 0);
            vga::print_boot_log("Mounting and initializing FAT16 file system driver", 0);
        }
        Err(e) => {
            let mut err_msg = [0u8; 80];
            let prefix = b"Mounting and initializing FAT16 file system driver (Error: ";
            let suffix = b")";
            let mut offset = 0;
            err_msg[offset..offset + prefix.len()].copy_from_slice(prefix);
            offset += prefix.len();
            let e_bytes = e.as_bytes();
            let to_copy = core::cmp::min(e_bytes.len(), err_msg.len() - offset - suffix.len());
            err_msg[offset..offset + to_copy].copy_from_slice(&e_bytes[..to_copy]);
            offset += to_copy;
            err_msg[offset..offset + suffix.len()].copy_from_slice(suffix);
            offset += suffix.len();
            if let Ok(msg_str) = core::str::from_utf8(&err_msg[..offset]) {
                vga::print_boot_log(msg_str, 1);
            } else {
                vga::print_boot_log("Mounting and initializing FAT16 file system driver", 1);
            }
        }
    }

    keira_mem::vmm::register_file_backing_hooks(file_backing_read, file_backing_sync);
    vga::print_boot_log(
        "Registering Virtual Memory demand paging and msync hooks",
        0,
    );
}
