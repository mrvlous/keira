// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'list' shell command.

use crate::args::CliArgs;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let args = CliArgs::parse(parts);

        if args.has_flag('h', "help") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: list [path] [-l] [-a] [-c]\n\n");
            vga::print_str(
                "Description:\n  List directory entries and files on FAT16 storage or Initrd.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str(
                "  -l, --long     Detailed tabular view with type, size, and attributes\n",
            );
            vga::print_str("  -a, --all      Show hidden files, system files, and dot entries\n");
            vga::print_str("  -c, --count    Print total entry count summary\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            return;
        }

        let show_all = args.has_flag('a', "all");
        let long_format = args.has_flag('l', "long");
        let show_count = args.has_flag('c', "count");
        let path_arg = args.first_positional();

        let target_cluster = if let Some(path) = path_arg {
            match keira_fs::fat::get_dir_cluster(path) {
                Ok(cluster) => cluster,
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("list: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            }
        } else {
            keira_fs::fat::CURRENT_DIR_CLUSTER
        };

        let mut total_files = 0;
        let mut total_dirs = 0;
        let mut total_bytes = 0;

        vga::set_color(vga::Color::White, vga::Color::Black);
        if long_format {
            vga::print_str("PERMS       TYPE     SIZE (BYTES)  NAME\n");
            vga::print_str("----------  -------  ------------  ------------------------\n");
        } else {
            vga::print_str("Directory of IDE disk:\n");
        }

        let res = keira_fs::fat::for_each_directory_entry(target_cluster, |parsed| {
            if let Ok(name_str) = core::str::from_utf8(&parsed.name[..parsed.name_len]) {
                if !show_all {
                    if name_str == "." || name_str == ".." {
                        return Ok(true);
                    }
                    if (parsed.entry.attr & 0x06) != 0 {
                        return Ok(true);
                    }
                }

                let is_dir = (parsed.entry.attr & 0x10) != 0;
                let size = parsed.entry.file_size as u64;

                if is_dir {
                    total_dirs += 1;
                } else {
                    total_files += 1;
                    total_bytes += size;
                }

                if long_format {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    if is_dir {
                        vga::print_str("drwxr-xr-x  [dir]    -             ");
                    } else {
                        vga::print_str("-rw-r--r--  [file]   ");
                        let mut s_buf = [0u8; 12];
                        let mut temp = size;
                        let mut s_idx = 12;
                        if temp == 0 {
                            s_idx -= 1;
                            s_buf[s_idx] = b'0';
                        } else {
                            while temp > 0 && s_idx > 0 {
                                s_idx -= 1;
                                s_buf[s_idx] = b'0' + (temp % 10) as u8;
                                temp /= 10;
                            }
                        }
                        for _ in 0..s_idx {
                            vga::print_str(" ");
                        }
                        if let Ok(s_str) = core::str::from_utf8(&s_buf[s_idx..]) {
                            vga::print_str(s_str);
                        }
                        vga::print_str("  ");
                    }
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str(name_str);
                    vga::print_str("\n");
                } else {
                    if is_dir {
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("  [dir]  ");
                        vga::print_str(name_str);
                    } else {
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("  [file] ");
                        vga::print_str(name_str);
                        vga::print_str(" (");
                        vga::print_u64(size);
                        vga::print_str(" bytes)");
                    }
                    vga::print_str("\n");
                }
            }
            Ok(true)
        });

        if res.is_err() {
            keira_fs::tar::list_files();
        } else if show_count || long_format {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("Total: ");
            vga::print_u64(total_files as u64);
            vga::print_str(" files (");
            vga::print_u64(total_bytes);
            vga::print_str(" bytes), ");
            vga::print_u64(total_dirs as u64);
            vga::print_str(" directories\n");
        }
    }
}
