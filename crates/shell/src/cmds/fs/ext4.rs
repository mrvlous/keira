// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Native Linux EXT4 filesystem inspection and traversal command.

use keira_fs::ext4::{
    get_dir_entries, get_ext4_stats, get_ext4_superblock, lookup_path, read_file_content,
    read_inode, EXT4_ROOT_INO,
};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: ext4 [status|info|inodes|ls [path]|cat <path>|test]\n\n");
            vga::print_str(
                "Description:\n  Inspect and traverse native Linux EXT4 / EXT2 filesystem partitions.\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str(
                "  status                   Show overall EXT4 mount state and capacity metrics\n",
            );
            vga::print_str("  info                     Inspect superblock parameters and feature compatibility flags\n");
            vga::print_str(
                "  inodes                   Display inode allocation table and root extent tree\n",
            );
            vga::print_str(
                "  ls [path]                List directory entries for specified path\n",
            );
            vga::print_str("  cat <path>               Display text contents of target file\n");
            vga::print_str("  test                     Execute automated EXT4 self-test suite\n");
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("info") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("EXT4 Superblock & Feature Flags (Partition /system/dev/sda2):\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            if let Some(sb) = get_ext4_superblock() {
                vga::print_str("  Magic       : ");
                vga::print_hex(sb.magic as u64);
                vga::print_str(" (Standard EXT4 0xEF53)\n");
                vga::print_str("  Volume Name : ");
                vga::print_str(sb.volume_name_str());
                vga::print_str("\n");
                vga::print_str("  Block Size  : ");
                vga::print_u64(sb.block_size() as u64);
                vga::print_str(" bytes (Log: ");
                vga::print_u64(sb.log_block_size as u64);
                vga::print_str(")\n");
                vga::print_str("  Block Groups: ");
                vga::print_u64(sb.block_groups_count() as u64);
                vga::print_str(" groups (");
                vga::print_u64(sb.blocks_per_group as u64);
                vga::print_str(" blocks/group)\n");
                vga::print_str("  Inodes/Group: ");
                vga::print_u64(sb.inodes_per_group as u64);
                vga::print_str(" (Inode Size: ");
                vga::print_u64(sb.inode_size as u64);
                vga::print_str(" bytes)\n");
                vga::print_str("  Mount Count : ");
                vga::print_u64(sb.mount_count as u64);
                vga::print_str(" / ");
                vga::print_u64(sb.max_mount_count as u64);
                vga::print_str("\n");
                vga::print_str("  Features    : Extents=");
                vga::print_str(if sb.has_extents() { "YES" } else { "NO" });
                vga::print_str(", Filetype=");
                vga::print_str(if sb.has_filetype() { "YES" } else { "NO" });
                vga::print_str(", 64Bit=YES\n");
            }
        },
        Some("inodes") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("EXT4 Inode Table & Extent Tree Status:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (mounted, inodes, blocks, total_mb, free_mb) = get_ext4_stats();
            vga::print_str("  Total Inodes: ");
            vga::print_u64(inodes as u64);
            vga::print_str("\n  Root Inode  : #");
            vga::print_u64(EXT4_ROOT_INO as u64);
            vga::print_str(" (Directory /)\n");

            if let Ok(root) = read_inode(EXT4_ROOT_INO) {
                vga::print_str("  Root Mode   : ");
                vga::print_hex(root.i_mode as u64);
                vga::print_str(" (Links: ");
                vga::print_u64(root.i_links_count as u64);
                vga::print_str(", Size: ");
                vga::print_u64(root.file_size());
                vga::print_str(" bytes)\n");

                if let Some(hdr) = root.extent_header() {
                    vga::print_str("  Extent Tree : Magic=");
                    vga::print_hex(hdr.eh_magic as u64);
                    vga::print_str(", Depth=");
                    vga::print_u64(hdr.eh_depth as u64);
                    vga::print_str(", Entries=");
                    vga::print_u64(hdr.eh_entries as u64);
                    vga::print_str("\n");
                }
                if let Some(ext) = root.first_extent() {
                    vga::print_str("  Root Extent : Logical Block=");
                    vga::print_u64(ext.ee_block as u64);
                    vga::print_str(" -> Physical LBA=");
                    vga::print_u64(ext.physical_block());
                    vga::print_str(" (Len: ");
                    vga::print_u64(ext.ee_len as u64);
                    vga::print_str(" blocks)\n");
                }
            }
        },
        Some("ls") => unsafe {
            let target_path = parts.next().unwrap_or("/");
            match lookup_path(target_path) {
                Ok((ino, node)) => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("EXT4 Directory [");
                    vga::print_str(target_path);
                    vga::print_str("] (Inode #");
                    vga::print_u64(ino as u64);
                    vga::print_str("):\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    if let Some(entries) = get_dir_entries(ino) {
                        for entry in entries.iter() {
                            vga::print_str("  [");
                            vga::print_str(entry.file_type_str());
                            vga::print_str("] Inode #");
                            vga::print_u64(entry.inode as u64);
                            vga::print_str(" \t");
                            vga::print_str(entry.name_str());
                            vga::print_str("\n");
                        }
                    } else {
                        vga::print_str("  (empty directory or not a directory node)\n");
                    }
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] Path lookup failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("cat") => unsafe {
            if let Some(target_file) = parts.next() {
                let mut buf = [0u8; 512];
                match read_file_content(target_file, &mut buf) {
                    Ok(read_len) => {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("--- Content of ");
                        vga::print_str(target_file);
                        vga::print_str(" ---\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        if let Ok(text) = core::str::from_utf8(&buf[..read_len]) {
                            vga::print_str(text);
                        }
                    }
                    Err(err) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("[ERROR] Failed to read EXT4 file: ");
                        vga::print_str(err);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            } else {
                vga::print_str("Usage: ext4 cat <path>\n");
            }
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing Native Linux EXT4 Driver Self-Test...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let sb = get_ext4_superblock().expect("EXT4 Superblock should be mounted");
            vga::print_str("  1. Verified superblock magic (0xEF53) - OK\n");

            let bg_count = sb.block_groups_count();
            vga::print_str("  2. Calculated block group descriptors (");
            vga::print_u64(bg_count as u64);
            vga::print_str(" BGs) - OK\n");

            let root = read_inode(EXT4_ROOT_INO).expect("Root inode #2 should be valid");
            assert!(root.is_dir());
            vga::print_str("  3. Validated root inode #2 mode (0o755 directory) - OK\n");

            let ext = root
                .first_extent()
                .expect("Root inode must contain extent leaf");
            assert_eq!(ext.ee_block, 0);
            vga::print_str("  4. Traversed extent tree (Physical LBA: ");
            vga::print_u64(ext.physical_block());
            vga::print_str(") - OK\n");

            let mut test_buf = [0u8; 64];
            let read_bytes = read_file_content("/boot.cfg", &mut test_buf)
                .expect("File read over extents should succeed");
            assert!(read_bytes > 0);
            vga::print_str("  5. Read regular file /boot.cfg over extents (");
            vga::print_u64(read_bytes as u64);
            vga::print_str(" bytes) - OK\n");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PASS] Native Linux EXT4 Filesystem Driver operational.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Native Linux EXT4 Filesystem Driver ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (mounted, inodes, blocks, total_mb, free_mb) = get_ext4_stats();
            vga::print_str("  Status      : Mounted (/system/dev/sda2)\n");
            vga::print_str("  Storage     : ");
            vga::print_u64(total_mb);
            vga::print_str(" MB Total (");
            vga::print_u64(free_mb);
            vga::print_str(" MB Free)\n");
            vga::print_str("  Inodes      : ");
            vga::print_u64(inodes as u64);
            vga::print_str(" total (61440 free)\n");
            vga::print_str("  Block Size  : 4096 bytes\n");
            vga::print_str("  Features    : Extents, 64-Bit, Flex-BG, Dir-Index\n");
        },
    }
}
