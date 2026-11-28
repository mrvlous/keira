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

//! Unit tests for Native EXT4 superblock, inode and directory parsing.

use super::*;

#[test]
fn test_ext4_superblock_defaults_and_calc() {
    let sb = Ext4Superblock::default();
    assert_eq!(sb.magic, EXT4_SUPER_MAGIC);
    assert_eq!(sb.block_size(), 4096);
    assert_eq!(sb.total_capacity_mb(), 1024);
    assert!(sb.has_extents());
    assert!(sb.has_filetype());
    assert_eq!(sb.volume_name_str(), "KEIRA_EXT4_ROOT");
}

#[test]
fn test_ext4_inode_validation_and_root() {
    assert!(validate_inode_num(2));
    assert!(validate_inode_num(11));
    assert!(!validate_inode_num(0));

    let root_inode = read_inode(2).expect("read root inode");
    assert!(root_inode.is_dir());
    assert!(root_inode.has_extents());
    assert_eq!(root_inode.file_size(), 4096);

    let ext = root_inode.first_extent().expect("root extent");
    assert_eq!(ext.ee_block, 0);
    assert_eq!(ext.ee_len, 1);
}

#[test]
fn test_ext4_dir_lookup() {
    let root_dot = lookup_in_dir(2, ".").expect("lookup .");
    assert_eq!(root_dot.inode, 2);

    let root_sys = lookup_in_dir(2, "system").expect("lookup system");
    assert_eq!(root_sys.inode, 11);
    assert_eq!(root_sys.file_type_str(), "dir");

    let sys_elf = lookup_in_dir(11, "kernel.elf").expect("lookup kernel.elf");
    assert_eq!(sys_elf.inode, 12);
    assert_eq!(sys_elf.file_type_str(), "file");

    assert!(lookup_in_dir(2, "nonexistent.bin").is_none());
}

struct MockExt4Disk;

static mut MOCK_EXT4_STORAGE: [[u8; 512]; 512] = [[0u8; 512]; 512];
static MOCK_EXT4_DISK: MockExt4Disk = MockExt4Disk;

impl keira_io::storage::block::BlockDevice for MockExt4Disk {
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        let idx = sector as usize;
        unsafe {
            if idx >= MOCK_EXT4_STORAGE.len() {
                return Err("MockExt4Disk: Sector out of bounds");
            }
            buffer.copy_from_slice(&MOCK_EXT4_STORAGE[idx]);
        }
        Ok(())
    }

    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        let idx = sector as usize;
        unsafe {
            if idx >= MOCK_EXT4_STORAGE.len() {
                return Err("MockExt4Disk: Sector out of bounds");
            }
            MOCK_EXT4_STORAGE[idx].copy_from_slice(buffer);
        }
        Ok(())
    }

    fn get_size_sectors(&self) -> u32 {
        512
    }

    fn get_name(&self) -> &'static str {
        "ext4"
    }
}

fn format_mock_ext4_image() {
    unsafe {
        MOCK_EXT4_STORAGE = [[0u8; 512]; 512];
    }

    let sb = Ext4Superblock {
        inodes_count: 64,
        blocks_count_lo: 64,
        r_blocks_count_lo: 0,
        free_blocks_count_lo: 53,
        free_inodes_count: 53,
        first_data_block: 0,
        log_block_size: 2,
        log_cluster_size: 2,
        blocks_per_group: 64,
        clusters_per_group: 64,
        inodes_per_group: 64,
        magic: EXT4_SUPER_MAGIC,
        state: EXT4_STATE_CLEAN,
        errors: 1,
        rev_level: 1,
        first_ino: 11,
        inode_size: 256,
        block_group_nr: 0,
        desc_size: 32,
        feature_compat: EXT4_FEATURE_COMPAT_DIR_INDEX,
        feature_incompat: EXT4_FEATURE_INCOMPAT_FILETYPE | EXT4_FEATURE_INCOMPAT_EXTENTS,
        feature_ro_compat: EXT4_FEATURE_RO_COMPAT_SPARSE_SUPER
            | EXT4_FEATURE_RO_COMPAT_LARGE_FILE
            | EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE,
        ..Default::default()
    };

    let mut raw_sb = [0u8; 1024];
    sb.write_to_bytes(&mut raw_sb);
    sb.patch_raw_bytes(&mut raw_sb);
    unsafe {
        MOCK_EXT4_STORAGE[2].copy_from_slice(&raw_sb[..512]);
        MOCK_EXT4_STORAGE[3].copy_from_slice(&raw_sb[512..]);
    }

    let bgd = Ext4GroupDesc {
        bg_block_bitmap_lo: 2,
        bg_inode_bitmap_lo: 3,
        bg_inode_table_lo: 4,
        bg_free_blocks_count_lo: 53,
        bg_free_inodes_count_lo: 53,
        bg_used_dirs_count_lo: 1,
    };
    let mut bgdt_block = [0u8; 4096];
    bgd.patch_bytes(&mut bgdt_block[..32]);
    write_mock_block(1, &bgdt_block);

    let mut bmp = [0u8; 4096];
    bmp[0] = 0xFF;
    bmp[1] = 0x07;
    write_mock_block(2, &bmp);
    write_mock_block(3, &bmp);

    let mut root_inode = Ext4Inode {
        i_mode: EXT4_S_IFDIR | 0o755,
        i_size_lo: 4096,
        i_links_count: 2,
        i_blocks_lo: 8,
        i_flags: EXT4_EXTENTS_FL,
        ..Default::default()
    };
    init_inode_extent_header(&mut root_inode);
    let hdr = Ext4ExtentHeader {
        eh_magic: EXT4_EXTENT_HEADER_MAGIC,
        eh_entries: 1,
        eh_max: 4,
        eh_depth: 0,
        eh_generation: 0,
    };
    let ext = Ext4Extent {
        ee_block: 0,
        ee_len: 1,
        ee_start_hi: 0,
        ee_start_lo: 10,
    };
    hdr.write_bytes(&mut root_inode.i_block[0..12]);
    ext.write_bytes(&mut root_inode.i_block[12..24]);

    let mut itbl_block = [0u8; 4096];
    write_inode_to_bytes(&root_inode, &mut itbl_block[256..512]);
    write_mock_block(4, &itbl_block);

    let mut root_dir = [0u8; 4096];
    let mut dot_name = [0u8; 32];
    dot_name[0] = b'.';
    let dot_entry = Ext4DirEntry {
        inode: 2,
        rec_len: 12,
        name_len: 1,
        file_type: EXT4_FT_DIR,
        name: dot_name,
    };
    let mut dotdot_name = [0u8; 32];
    dotdot_name[..2].copy_from_slice(b"..");
    let dotdot_entry = Ext4DirEntry {
        inode: 2,
        rec_len: 4084,
        name_len: 2,
        file_type: EXT4_FT_DIR,
        name: dotdot_name,
    };
    write_dir_entry_to_block(&dot_entry, &mut root_dir, 0);
    write_dir_entry_to_block(&dotdot_entry, &mut root_dir, 12);
    write_mock_block(10, &root_dir);
}

fn write_mock_block(blk_idx: usize, data: &[u8; 4096]) {
    let base_sec = blk_idx * 8;
    unsafe {
        for s in 0..8 {
            MOCK_EXT4_STORAGE[base_sec + s].copy_from_slice(&data[s * 512..(s + 1) * 512]);
        }
    }
}

#[test]
fn test_ext4_rw_kflushd_extents_and_unlink() {
    format_mock_ext4_image();
    let _ = keira_io::storage::block::register_device(&MOCK_EXT4_DISK);
    init_with_device("ext4").expect("mount mock ext4 device");

    let (_, _, _, _, _) = get_ext4_stats();
    let initial_sb = get_ext4_superblock().expect("active sb");
    assert_eq!(initial_sb.free_blocks_count_lo, 53);
    assert_eq!(initial_sb.free_inodes_count, 53);

    ext4_create_dir("/work").expect("create dir");
    assert!(ext4_is_dir("/work"));
    assert!(ext4_dir_has_child("/", "work"));

    ext4_create_file("/work/hello.txt").expect("create file");
    assert!(ext4_exists("/work/hello.txt"));

    let payload = b"Keira EXT4 read-write page cache persistence verification!";
    let written = ext4_write_file("/work/hello.txt", payload).expect("write file");
    assert_eq!(written, payload.len());
    assert!(crate::cache::count_dirty_pages() > 0);

    let mut read_back = [0u8; 128];
    let n = ext4_read_file("/work/hello.txt", &mut read_back).expect("read cached file");
    assert_eq!(&read_back[..n], payload);

    let flushed = unsafe { crate::cache::kflushd_step() };
    assert!(flushed >= 1);
    assert_eq!(crate::cache::count_dirty_pages(), 0);

    // Remount to confirm bytes persisted onto physical block array rather than remaining in RAM.
    init_with_device("ext4").expect("remount mock ext4");
    let mut disk_back = [0u8; 128];
    let n2 = ext4_read_file("/work/hello.txt", &mut disk_back).expect("read after remount");
    assert_eq!(&disk_back[..n2], payload);

    // Multi-block write spanning 3 contiguous blocks (9 KiB) with extent coalescing + flush.
    let mut multi = [0u8; 9000];
    for (i, b) in multi.iter_mut().enumerate() {
        *b = ((i * 7 + 3) & 0xFF) as u8;
    }
    ext4_create_file("/work/multi.bin").expect("create multi");
    ext4_write_file("/work/multi.bin", &multi).expect("write multi");
    let flushed_multi = unsafe { crate::cache::kflushd_step() };
    assert!(flushed_multi >= 3);

    let mut check_multi = [0u8; 9000];
    let nm = ext4_read_file("/work/multi.bin", &mut check_multi).expect("read multi");
    assert_eq!(nm, multi.len());
    assert_eq!(&check_multi[..nm], &multi[..]);

    ext4_remove_entry("/work/multi.bin").expect("remove multi.bin");
    ext4_remove_entry("/work/hello.txt").expect("remove hello.txt");
    ext4_remove_entry("/work").expect("remove dir work");
    sync_filesystem().expect("sync filesystem");

    let final_sb = get_ext4_superblock().expect("final sb");
    assert_eq!(final_sb.free_blocks_count_lo, 53);
    assert_eq!(final_sb.free_inodes_count, 53);
}
