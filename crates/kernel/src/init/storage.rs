// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Storage block device discovery, FAT16 filesystem mounting and VMM demand paging.

use keira_fs::fat;
use keira_io::bus::pci;
use keira_io::storage::{ahci, block, ide};

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

/// Initialize PCI host controllers, storage devices, root FAT16 filesystem and VMM hooks.
///
/// # Safety
/// Caller guarantees virtual memory and scheduler primitives are active.
pub unsafe fn init_storage_and_fs() {
    keira_core::pr_info!("pci", "PCI bus enumeration and host controller discovery");
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

    if mounted {
        match fat::init() {
            Ok(_) => {
                if let Some(dev) = block::get_mounted_device() {
                    if dev.get_name() == "ahci0" {
                        keira_core::pr_info!(
                            "ahci",
                            "SATA AHCI host controller detected and active"
                        );
                    } else {
                        keira_core::pr_info!("ide", "IDE primary master drive detected");
                    }
                }
                keira_core::pr_info!("block", "active block device registered");
                keira_core::pr_info!("vfs", "mounted root filesystem (FAT16)");
            }
            Err(e) => {
                keira_core::pr_warn!("vfs", "FAT16 root mount failed: {}", e);
            }
        }
    } else if keira_fs::tar::is_initrd_loaded() {
        keira_core::pr_info!(
            "block",
            "no hardware drive attached; falling back to ramdisk"
        );
        keira_core::pr_info!("vfs", "mounted root filesystem (ramdisk/initrd)");
    } else {
        keira_core::pr_info!(
            "block",
            "no hardware drive or initrd attached; virtual rootfs active (procfs/devfs/tmpfs)"
        );
    }

    if block::get_device("ahci1").is_some() {
        if !keira_fs::vfs::exists("/mnt") {
            let _ = keira_fs::vfs::create_dir("/mnt");
        }
        if !keira_fs::vfs::exists("/mnt/ext4") {
            let _ = keira_fs::vfs::create_dir("/mnt/ext4");
        }
        match keira_fs::vfs::vfs_mount(
            "/dev/sdb",
            "/mnt/ext4",
            keira_fs::vfs::FilesystemType::Ext4,
            0,
        ) {
            Ok(()) => {
                keira_core::pr_info!(
                    "ext4",
                    "mounted secondary EXT4 read-write volume on /mnt/ext4 (/dev/sdb)"
                );
            }
            Err(e) => {
                keira_core::pr_warn!("ext4", "secondary /dev/sdb EXT4 mount skipped: {}", e);
            }
        }
    }

    keira_mem::vmm::register_file_backing_hooks(file_backing_read, file_backing_sync);
    keira_core::pr_info!(
        "vmm",
        "demand paging and msync file backing hooks registered"
    );
}

pub mod mount {
    pub use super::init_storage_and_fs;
}
