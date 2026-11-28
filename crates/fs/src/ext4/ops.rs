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

//! High-level VFS operations and [kflushd] writeback integration for Read-Write EXT4.

use crate::cache::{
    find_page_index, insert_page, invalidate_inode, read_page, write_page, EXT4_CACHE_DEV_ID,
    PAGE_SIZE,
};
use crate::ext4::bitmap::{alloc_inode, free_inode};
use crate::ext4::dir::{
    disk_add_dir_entry, disk_get_dir_entry, disk_lookup_in_dir, disk_lookup_path,
    disk_remove_dir_entry, split_parent_and_name, Ext4DirTuple, EXT4_FT_DIR, EXT4_FT_REG_FILE,
};
use crate::ext4::inode::{
    alloc_and_map_block_disk, free_all_extents_disk, init_inode_extent_header, read_inode_disk,
    resolve_logical_block_disk, write_inode_disk, Ext4Inode, EXT4_EXTENTS_FL, EXT4_S_IFDIR,
    EXT4_S_IFREG,
};
use crate::ext4::superblock::{
    get_ext4_superblock, is_disk_mounted, read_block_4k, sync_superblock, write_block_4k,
};

fn ensure_ready() -> Result<(), &'static str> {
    if !is_disk_mounted() {
        let _ = crate::ext4::superblock::init();
    }
    if !is_disk_mounted() {
        return Err("EXT4: No physical EXT4 block device mounted");
    }
    Ok(())
}

/// Check whether `path` exists in the mounted EXT4 filesystem.
pub fn ext4_exists(path: &str) -> bool {
    if ensure_ready().is_err() {
        return false;
    }
    disk_lookup_path(path).is_ok()
}

/// Check whether `path` is a directory in the mounted EXT4 filesystem.
pub fn ext4_is_dir(path: &str) -> bool {
    let clean = path.trim_matches('/');
    if ensure_ready().is_err() {
        return clean.is_empty();
    }
    match disk_lookup_path(clean) {
        Ok((_, _, inode)) => inode.is_dir(),
        Err(_) => false,
    }
}

/// Query file size in bytes for `path` in the mounted EXT4 filesystem.
pub fn ext4_get_file_size(path: &str) -> Result<usize, &'static str> {
    ensure_ready()?;
    let (_, _, inode) = disk_lookup_path(path)?;
    Ok(inode.file_size() as usize)
}

/// Create a regular empty file at `path` inside the mounted EXT4 filesystem.
pub fn ext4_create_file(path: &str) -> Result<(), &'static str> {
    ensure_ready()?;
    if disk_lookup_path(path).is_ok() {
        return Err("EXT4: File already exists");
    }
    let (parent_path, name) = split_parent_and_name(path)?;
    let (_, parent_ino, _) = disk_lookup_path(parent_path)?;

    let child_ino = alloc_inode(false)?;
    let mut child = Ext4Inode {
        i_mode: EXT4_S_IFREG | 0o644,
        i_size_lo: 0,
        i_links_count: 1,
        i_blocks_lo: 0,
        i_flags: EXT4_EXTENTS_FL,
        ..Default::default()
    };
    init_inode_extent_header(&mut child);
    write_inode_disk(child_ino, &child)?;

    if let Err(e) = disk_add_dir_entry(parent_ino, child_ino, EXT4_FT_REG_FILE, name) {
        let _ = free_inode(child_ino, false);
        return Err(e);
    }
    Ok(())
}

/// Create a directory at `path` with `.` and `..` entries inside the mounted EXT4 filesystem.
pub fn ext4_create_dir(path: &str) -> Result<(), &'static str> {
    ensure_ready()?;
    if disk_lookup_path(path).is_ok() {
        return Err("EXT4: Directory already exists");
    }
    let (parent_path, name) = split_parent_and_name(path)?;
    let (_, parent_ino, mut parent_inode) = disk_lookup_path(parent_path)?;

    let child_ino = alloc_inode(true)?;
    let mut child = Ext4Inode {
        i_mode: EXT4_S_IFDIR | 0o755,
        i_size_lo: 4096,
        i_links_count: 2,
        i_blocks_lo: 0,
        i_flags: EXT4_EXTENTS_FL,
        ..Default::default()
    };
    init_inode_extent_header(&mut child);
    let dir_phys = alloc_and_map_block_disk(child_ino, &mut child, 0)?;

    let mut blk = [0u8; 4096];
    blk[0..4].copy_from_slice(&child_ino.to_le_bytes());
    blk[4..6].copy_from_slice(&12u16.to_le_bytes());
    blk[6] = 1;
    blk[7] = EXT4_FT_DIR;
    blk[8] = b'.';

    blk[12..16].copy_from_slice(&parent_ino.to_le_bytes());
    blk[16..18].copy_from_slice(&4084u16.to_le_bytes());
    blk[18] = 2;
    blk[19] = EXT4_FT_DIR;
    blk[20] = b'.';
    blk[21] = b'.';
    write_block_4k(dir_phys, &blk)?;

    if let Err(e) = disk_add_dir_entry(parent_ino, child_ino, EXT4_FT_DIR, name) {
        let _ = free_all_extents_disk(&mut child);
        let _ = free_inode(child_ino, true);
        return Err(e);
    }

    parent_inode.i_links_count = parent_inode.i_links_count.saturating_add(1);
    write_inode_disk(parent_ino, &parent_inode)
}

/// Unlink a regular file or empty directory at `path` and reclaim its extents and inode.
pub fn ext4_remove_entry(path: &str) -> Result<(), &'static str> {
    ensure_ready()?;
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Err("EXT4: Cannot remove root directory");
    }
    let (_, target_ino, mut target_inode) = disk_lookup_path(path)?;
    let is_dir = target_inode.is_dir();

    if is_dir {
        let mut idx = 0usize;
        let mut child_count = 0usize;
        while let Some((_, _, nbuf, nlen)) = disk_get_dir_entry(target_ino, idx)? {
            if let Ok(name) = core::str::from_utf8(&nbuf[..nlen]) {
                if name != "." && name != ".." {
                    child_count += 1;
                    break;
                }
            }
            idx += 1;
        }
        if child_count > 0 {
            return Err("EXT4: Directory not empty");
        }
    }

    let (parent_path, name) = split_parent_and_name(path)?;
    let (_, parent_ino, mut parent_inode) = disk_lookup_path(parent_path)?;

    unsafe {
        invalidate_inode(EXT4_CACHE_DEV_ID, target_ino);
    }

    disk_remove_dir_entry(parent_ino, name)?;

    if is_dir {
        parent_inode.i_links_count = parent_inode.i_links_count.saturating_sub(1);
        let _ = write_inode_disk(parent_ino, &parent_inode);
    }

    target_inode.i_links_count = target_inode.i_links_count.saturating_sub(1);
    if target_inode.i_links_count == 0 || is_dir {
        free_all_extents_disk(&mut target_inode)?;
        let _ = write_inode_disk(target_ino, &target_inode);
        free_inode(target_ino, is_dir)?;
    } else {
        let _ = write_inode_disk(target_ino, &target_inode);
    }

    Ok(())
}

/// Read up to `buf.len()` bytes from EXT4 file `path` starting at `offset` via page cache.
pub fn ext4_read_file_offset(
    path: &str,
    offset: u64,
    buf: &mut [u8],
) -> Result<usize, &'static str> {
    ensure_ready()?;
    let (_, ino, inode) = disk_lookup_path(path)?;
    if !inode.is_regular_file() {
        return Err("EXT4: Target is not a regular file");
    }

    let file_size = inode.file_size();
    if offset >= file_size || buf.is_empty() {
        return Ok(0);
    }

    let total_to_read = core::cmp::min(buf.len() as u64, file_size - offset) as usize;
    let mut read_done = 0usize;
    let mut blk_buf = [0u8; PAGE_SIZE];

    while read_done < total_to_read {
        let pos = (offset as usize) + read_done;
        let page_idx = (pos / PAGE_SIZE) as u32;
        let page_off = pos % PAGE_SIZE;
        let chunk = core::cmp::min(PAGE_SIZE - page_off, total_to_read - read_done);

        unsafe {
            if find_page_index(EXT4_CACHE_DEV_ID, ino, page_idx).is_none() {
                blk_buf.fill(0);
                if let Some(phys) = resolve_logical_block_disk(&inode, page_idx)? {
                    read_block_4k(phys, &mut blk_buf)?;
                }
                let page_valid = core::cmp::min(
                    PAGE_SIZE,
                    (file_size as usize).saturating_sub((page_idx as usize) * PAGE_SIZE),
                );
                let _ = insert_page(
                    EXT4_CACHE_DEV_ID,
                    ino,
                    page_idx,
                    &blk_buf[..page_valid.max(page_off + chunk).min(PAGE_SIZE)],
                    false,
                );
            }

            if let Some(n) = read_page(
                EXT4_CACHE_DEV_ID,
                ino,
                page_idx,
                page_off,
                &mut buf[read_done..read_done + chunk],
            ) {
                if n == 0 {
                    buf[read_done..read_done + chunk].fill(0);
                }
            }
        }

        read_done += chunk;
    }

    Ok(read_done)
}

/// Read file `path` from byte 0 into `buf`.
pub fn ext4_read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    ext4_read_file_offset(path, 0, buf)
}

/// Write `content` at `offset` into EXT4 file `path` staging dirty pages in unified cache.
pub fn ext4_write_file_offset(
    path: &str,
    offset: u64,
    content: &[u8],
) -> Result<usize, &'static str> {
    ensure_ready()?;
    if disk_lookup_path(path).is_err() {
        ext4_create_file(path)?;
    }

    let (_, ino, mut inode) = disk_lookup_path(path)?;
    if !inode.is_regular_file() {
        return Err("EXT4: Target is not a regular file");
    }

    let mut written = 0usize;
    let mut blk_buf = [0u8; PAGE_SIZE];

    while written < content.len() {
        let pos = (offset as usize) + written;
        let page_idx = (pos / PAGE_SIZE) as u32;
        let page_off = pos % PAGE_SIZE;
        let chunk = core::cmp::min(PAGE_SIZE - page_off, content.len() - written);

        unsafe {
            if find_page_index(EXT4_CACHE_DEV_ID, ino, page_idx).is_none()
                && (page_off > 0 || chunk < PAGE_SIZE)
            {
                if let Some(phys) = resolve_logical_block_disk(&inode, page_idx)? {
                    read_block_4k(phys, &mut blk_buf)?;
                    let cur_valid = core::cmp::min(
                        PAGE_SIZE,
                        (inode.file_size() as usize)
                            .saturating_sub((page_idx as usize) * PAGE_SIZE),
                    );
                    let _ = insert_page(
                        EXT4_CACHE_DEV_ID,
                        ino,
                        page_idx,
                        &blk_buf[..cur_valid],
                        false,
                    );
                }
            }

            let _ = write_page(
                EXT4_CACHE_DEV_ID,
                ino,
                page_idx,
                page_off,
                &content[written..written + chunk],
            )?;
        }

        written += chunk;
    }

    let new_end = offset.saturating_add(written as u64);
    if new_end > inode.file_size() {
        inode.i_size_lo = (new_end & 0xFFFF_FFFF) as u32;
        inode.i_size_high = ((new_end >> 32) & 0xFFFF_FFFF) as u32;
        write_inode_disk(ino, &inode)?;
    }

    Ok(written)
}

/// Truncate or overwrite EXT4 regular file `path` with `content` via dirty page cache.
pub fn ext4_write_file(path: &str, content: &[u8]) -> Result<usize, &'static str> {
    ensure_ready()?;
    if disk_lookup_path(path).is_err() {
        ext4_create_file(path)?;
    } else {
        let (_, ino, mut inode) = disk_lookup_path(path)?;
        if inode.file_size() > content.len() as u64 {
            unsafe {
                invalidate_inode(EXT4_CACHE_DEV_ID, ino);
            }
            free_all_extents_disk(&mut inode)?;
            inode.i_size_lo = 0;
            inode.i_size_high = 0;
            write_inode_disk(ino, &inode)?;
        }
    }

    let written = ext4_write_file_offset(path, 0, content)?;
    let (_, ino, mut inode) = disk_lookup_path(path)?;
    if inode.file_size() != content.len() as u64 {
        inode.i_size_lo = (content.len() as u64 & 0xFFFF_FFFF) as u32;
        inode.i_size_high = ((content.len() as u64 >> 32) & 0xFFFF_FFFF) as u32;
        write_inode_disk(ino, &inode)?;
    }
    Ok(written)
}

/// Flush one dirty 4 KiB page from page cache back onto physical EXT4 disk blocks.
pub fn writeback_page(inode_num: u32, page_index: u32, data: &[u8]) -> Result<(), &'static str> {
    ensure_ready()?;
    let mut inode = read_inode_disk(inode_num)?;
    let phys_block = alloc_and_map_block_disk(inode_num, &mut inode, page_index)?;

    let mut blk_buf = [0u8; 4096];
    let copy_len = data.len().min(4096);
    blk_buf[..copy_len].copy_from_slice(&data[..copy_len]);
    write_block_4k(phys_block, &blk_buf)?;

    let end_byte = ((page_index as u64) * 4096).saturating_add(copy_len as u64);
    if end_byte > inode.file_size() {
        inode.i_size_lo = (end_byte & 0xFFFF_FFFF) as u32;
        inode.i_size_high = ((end_byte >> 32) & 0xFFFF_FFFF) as u32;
        write_inode_disk(inode_num, &inode)?;
    }

    Ok(())
}

/// Synchronize dirty EXT4 superblock metadata and flush underlying storage device caches.
pub fn sync_filesystem() -> Result<(), &'static str> {
    sync_superblock()
}

/// Retrieve `(inode_num, d_type, name_buf, name_len)` for directory `path` entry `index`.
pub fn ext4_get_dir_entry_by_path(
    path: &str,
    index: usize,
) -> Result<Option<Ext4DirTuple>, &'static str> {
    ensure_ready()?;
    let (_, dir_ino, _) = disk_lookup_path(path)?;
    disk_get_dir_entry(dir_ino, index)
}

/// Verify whether directory `path` contains a child entry named `child` on disk.
pub fn ext4_dir_has_child(path: &str, child: &str) -> bool {
    if ensure_ready().is_err() {
        return false;
    }
    if let Ok((_, dir_ino, _)) = disk_lookup_path(path) {
        return disk_lookup_in_dir(dir_ino, child).ok().flatten().is_some();
    }
    false
}

/// Return `(block_size, total_blocks, free_blocks)` telemetry for mounted EXT4 volume.
pub fn ext4_statfs() -> (u64, u64, u64) {
    if let Some(sb) = get_ext4_superblock() {
        (
            sb.block_size() as u64,
            sb.blocks_count_lo as u64,
            sb.free_blocks_count_lo as u64,
        )
    } else {
        (4096, 0, 0)
    }
}
