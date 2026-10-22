// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Ring 0 supervisor mount command for inspecting and mutating VFS mount points.

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
