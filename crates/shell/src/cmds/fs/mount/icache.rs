// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'icache' supervisor diagnostic shell command.

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
            vga::print_str("  -c, --clear   Flush and invalidate all cached inode descriptors\n");
            vga::print_str("  -f, --flush   Flush dirty cached inodes to storage writeback\n");
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
