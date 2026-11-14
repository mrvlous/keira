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

//! Filesystem and storage mount shell commands.

pub mod mount {
    pub mod dcache {
        use core::mem::size_of;
        use keira_fs::cache::{dcache_clear, dcache_stats, Dentry, DENTRY_CACHE_CAPACITY};
        use keira_io::vga;

        /// Executes the 'dcache' command displaying VFS in-memory dentry cache telemetry.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let bg = vga::Color::Black;
            let fg = vga::Color::White;

            if let Some(arg) = parts.next() {
                if arg == "-h" || arg == "--help" {
                    vga::set_color(fg, bg);
                    vga::print_str("Usage: dcache [-h] [-c]\n\n");
                    vga::print_str("Description:\n  Inspect VFS in-memory dentry cache (dcache) telemetry and hit ratios.\n\n");
                    vga::print_str("Options:\n  -c, --clear   Flush and invalidate all cached directory entries\n  -h, --help    Show this help reference and exit\n");
                    return;
                }

                if arg == "-c" || arg == "--clear" {
                    unsafe {
                        dcache_clear();
                    }
                    vga::set_color(vga::Color::LightGreen, bg);
                    vga::print_str("[dcache] In-memory dentry cache successfully flushed.\n");
                    vga::set_color(fg, bg);
                    return;
                }
            }

            let (hits, misses, neg_hits, evictions, active_pos, active_neg) = dcache_stats();
            let total_lookups = hits.saturating_add(misses).saturating_add(neg_hits);
            let total_hits = hits.saturating_add(neg_hits);
            let hit_pct = total_hits
                .saturating_mul(100)
                .checked_div(total_lookups)
                .unwrap_or(0);
            let total_active = active_pos.saturating_add(active_neg);
            let memory_used = total_active.saturating_mul(size_of::<Dentry>());

            vga::set_color(fg, bg);
            vga::print_str("VFS Dentry Cache (dcache) Telemetry:\n");

            vga::print_str("  Total Capacity:     ");
            print_num(DENTRY_CACHE_CAPACITY as u64);
            vga::print_str(" slots\n");

            vga::print_str("  Active Entries:     ");
            print_num(total_active as u64);
            vga::print_str(" (positive: ");
            print_num(active_pos as u64);
            vga::print_str(", negative: ");
            print_num(active_neg as u64);
            vga::print_str(")\n");

            vga::print_str("  Memory Footprint:   ");
            print_num(memory_used as u64);
            vga::print_str(" bytes\n");

            vga::print_str("  Cache Hits:         ");
            print_num(hits);
            vga::print_str(" (negative hits: ");
            print_num(neg_hits);
            vga::print_str(")\n");

            vga::print_str("  Cache Misses:       ");
            print_num(misses);
            vga::print_str("\n");

            vga::print_str("  LRU Evictions:      ");
            print_num(evictions);
            vga::print_str("\n");

            vga::print_str("  Lookup Hit Ratio:   ");
            print_num(hit_pct);
            vga::print_str("%\n");
        }

        fn print_num(val: u64) {
            if val == 0 {
                vga::print_str("0");
                return;
            }
            let mut buf = [0u8; 20];
            let mut i = 0;
            let mut v = val;
            while v > 0 {
                buf[i] = b'0' + (v % 10) as u8;
                v /= 10;
                i += 1;
            }
            buf[..i].reverse();
            if let Ok(s) = core::str::from_utf8(&buf[..i]) {
                vga::print_str(s);
            }
        }
    }
    pub mod disk {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            unsafe {
                if let Some("-h") | Some("--help") = parts.next() {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: disk [-h]\n\n");
                    vga::print_str("Description:\n  Display primary storage drive geometry, sector layout and active FAT filesystem details.\n\n");
                    vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                keira_fs::fat::print_disk_info();
            }
        }
    }
    pub mod icache {
        use core::mem::size_of;
        use keira_fs::cache::{
            icache_clear, icache_flush_dirty, icache_stats, CachedInode, INODE_CACHE_CAPACITY,
        };
        use keira_io::vga;

        /// Executes the 'icache' command displaying VFS in-memory inode cache telemetry.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let bg = vga::Color::Black;
            let fg = vga::Color::White;

            if let Some(arg) = parts.next() {
                if arg == "-h" || arg == "--help" {
                    vga::set_color(fg, bg);
                    vga::print_str("Usage: icache [-h] [-c] [-f]\n\n");
                    vga::print_str(
                "Description:\n  Inspect VFS in-memory inode cache (icache) telemetry, dirty status and hit ratios.\n\n",
            );
                    vga::print_str("Options:\n");
                    vga::print_str(
                        "  -c, --clear   Flush and invalidate all cached inode descriptors\n",
                    );
                    vga::print_str(
                        "  -f, --flush   Flush dirty cached inodes to storage writeback\n",
                    );
                    vga::print_str("  -h, --help    Show this help reference and exit\n");
                    vga::set_color(vga::Color::LightGrey, bg);
                    return;
                }

                if arg == "-c" || arg == "--clear" {
                    icache_clear();
                    vga::set_color(vga::Color::LightGreen, bg);
                    vga::print_str("[icache] In-memory inode cache successfully cleared.\n");
                    vga::set_color(fg, bg);
                    return;
                }

                if arg == "-f" || arg == "--flush" {
                    let count = icache_flush_dirty();
                    vga::set_color(vga::Color::LightGreen, bg);
                    vga::print_str("[icache] Flushed ");
                    print_num(count as u64);
                    vga::print_str(" dirty inodes to storage writeback.\n");
                    vga::set_color(fg, bg);
                    return;
                }
            }

            let (hits, misses, evictions, writebacks, active, dirty) = icache_stats();
            let total_lookups = hits.saturating_add(misses);
            let hit_pct = hits
                .saturating_mul(100)
                .checked_div(total_lookups)
                .unwrap_or(0);
            let memory_used = active.saturating_mul(size_of::<CachedInode>());

            vga::set_color(fg, bg);
            vga::print_str("VFS Inode Cache (icache) Telemetry:\n");

            vga::print_str("  Total Capacity:     ");
            print_num(INODE_CACHE_CAPACITY as u64);
            vga::print_str(" slots\n");

            vga::print_str("  Active Inodes:      ");
            print_num(active as u64);
            vga::print_str(" descriptors\n");

            vga::print_str("  Dirty Inodes:       ");
            print_num(dirty as u64);
            vga::print_str(" uncommitted\n");

            vga::print_str("  Cache Hits:         ");
            print_num(hits);
            vga::print_str(" lookups\n");

            vga::print_str("  Cache Misses:       ");
            print_num(misses);
            vga::print_str(" lookups\n");

            vga::print_str("  Hit Ratio:          ");
            print_num(hit_pct);
            vga::print_str("%\n");

            vga::print_str("  Clock Evictions:    ");
            print_num(evictions);
            vga::print_str(" slots\n");

            vga::print_str("  Dirty Writebacks:   ");
            print_num(writebacks);
            vga::print_str(" synchronizations\n");

            vga::print_str("  Resident Memory:    ");
            print_num(memory_used as u64);
            vga::print_str(" bytes\n");

            vga::set_color(vga::Color::LightGrey, bg);
        }

        fn print_num(val: u64) {
            vga::print_u64(val);
        }
    }
    pub mod mount {
        use crate::args::CliArgs;
        use keira_fs::vfs::{get_active_mounts, vfs_mount, FilesystemType};
        use keira_io::vga;

        /// Entry point for supervisor `mount` command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: mount [-h] [-t fstype] [device] [target]\n\n");
                vga::print_str("Description:\n  Display active VFS mount table or register a new filesystem mount point.\n\n");
                vga::print_str("Options:\n  -h, --help        Show this help message and exit\n  -t, --type TYPE   Filesystem type (fat, proc, dev, tmpfs, ext4)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let pos0 = args.positional(0);
            let pos1 = args.positional(1);

            if pos0.is_none() {
                let (mounts, count) = get_active_mounts();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Active VFS Filesystem Mount Points:\n\n");
                vga::print_str("  DEVICE               MOUNT POINT          TYPE      OPTIONS\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                for m in mounts[..count].iter() {
                    vga::print_str("  ");
                    let src = m.source_str();
                    vga::print_str(src);
                    for _ in 0..21usize.saturating_sub(src.len()) {
                        vga::print_str(" ");
                    }

                    let tgt = m.target_str();
                    vga::print_str(tgt);
                    for _ in 0..21usize.saturating_sub(tgt.len()) {
                        vga::print_str(" ");
                    }

                    let fs = m.fs_type.as_str();
                    vga::print_str(fs);
                    for _ in 0..10usize.saturating_sub(fs.len()) {
                        vga::print_str(" ");
                    }

                    let mut flags_buf = [0u8; 64];
                    let flen = m.format_flags(&mut flags_buf);
                    if let Ok(flg) = core::str::from_utf8(&flags_buf[..flen]) {
                        vga::print_str(flg);
                    }
                    vga::print_str("\n");
                }
                return;
            }

            let fstype_str = args.get_opt('t', "type").unwrap_or("fat");
            let fs_type = match FilesystemType::from_str(fstype_str) {
                Some(t) => t,
                None => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("mount: unknown filesystem type: ");
                    vga::print_str(fstype_str);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            };

            let (source, target) = if let (Some(s), Some(t)) = (pos0, pos1) {
                (s, t)
            } else if let Some(t) = pos0 {
                ("none", t)
            } else {
                ("none", "/")
            };

            match vfs_mount(source, target, fs_type, 0) {
                Ok(()) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Mounted ");
                    vga::print_str(source);
                    vga::print_str(" on ");
                    vga::print_str(target);
                    vga::print_str(" (type ");
                    vga::print_str(fs_type.as_str());
                    vga::print_str(")\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("mount failed: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
    }
}

pub use self::mount::dcache;
pub use self::mount::disk;
pub use self::mount::icache;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_commands_help_invocation() {
        let mut args = "disk --help".split_whitespace();
        args.next();
        disk::run(&mut args);

        let mut dcache_args = "dcache --help".split_whitespace();
        dcache_args.next();
        dcache::run(&mut dcache_args);

        let mut dcache_run_args = "dcache".split_whitespace();
        dcache_run_args.next();
        dcache::run(&mut dcache_run_args);

        let mut icache_help_args = "icache --help".split_whitespace();
        icache_help_args.next();
        icache::run(&mut icache_help_args);

        let mut icache_clear_args = "icache -c".split_whitespace();
        icache_clear_args.next();
        icache::run(&mut icache_clear_args);

        let mut icache_flush_args = "icache -f".split_whitespace();
        icache_flush_args.next();
        icache::run(&mut icache_flush_args);

        let mut icache_run_args = "icache".split_whitespace();
        icache_run_args.next();
        icache::run(&mut icache_run_args);
    }
}
