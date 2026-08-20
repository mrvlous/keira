// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Filesystem abstraction layer (VFS, FAT16, Ext4, Tar/Initrd, ELF64, Character Devices, Locks, LVM).

pub mod dev;
pub mod elf;
pub mod ext4;
pub mod fat;
pub mod lock;
pub mod lvm;
pub mod tar;
pub mod vfs;

pub use dev::char::{read_dev_node, write_dev_node};
pub use elf::loader::{execute_user_mode, load_elf};
pub use elf::types::{ElfHeader, ProgramHeader, PT_LOAD};
pub use ext4::inode::{read_inode, validate_inode_num};
pub use ext4::superblock::{init as ext4_init, Ext4Superblock, EXT4_SUPER_MAGIC, MOUNTED_EXT4};
pub use fat::cluster::{alloc_cluster, fat_next_cluster, free_cluster_chain};
pub use fat::dir::{
    change_directory, create_dir, create_directory_entry, create_directory_entry_with_name,
    find_matches, get_dir_cluster, get_rtc_fat_time_date, init_dir_cluster, is_dir_empty,
    lfn_checksum, list_files, list_files_in_dir, ParsedDirectoryEntry,
};
pub use fat::file::{
    append_file_content, cat_file, create_file as fat_create_file,
    read_file_content as fat_read_file_content, remove_entry as fat_remove_entry,
    write_file_content as fat_write_file_content,
};
pub use fat::path::{filename_to_8_3, find_entry, format_filename, resolve_path, sanitize_path};
pub use fat::table::{
    clear_cache, flush_dirty_sectors, read_sector as fat_read_sector,
    write_sector as fat_write_sector,
};
pub use fat::types::{DirectoryEntry, Fat16Volume, FoundEntry, LfnAccumulator, LfnEntry};
pub use fat::volume::{
    cluster_to_sector, init as fat_init, print_disk_info, CURRENT_DIR_CLUSTER, VOLUME,
};
pub use lock::flock::{
    acquire_lock, release_all_locks_for_task, release_lock, FileLock, FILE_LOCKS, MAX_FILE_LOCKS,
};
pub use lvm::volume::{
    sys_raid_lvm, LogicalVolume, PhysicalVolume, RaidArray, VolumeGroup, LVM_CMD_CREATE_LV,
    LVM_CMD_CREATE_VG, LVM_CMD_INFO, LVM_CMD_RAID_STATUS, LVM_CMD_RAID_SYNC,
};
pub use tar::reader::{
    cat_file as tar_cat_file, exists as tar_exists, init as tar_init, list_files as tar_list_files,
    read_file_content as tar_read_file_content,
};
pub use vfs::ops::{
    create_dir as vfs_create_dir, create_file, exists, read_file, remove_entry, write_file,
};
pub use vfs::path::{resolve_alias_path, route_path};
pub use vfs::permissions::{check_access_permission, get_vfs_user, set_vfs_user};
pub use vfs::types::FilesystemType;
