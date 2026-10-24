// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'dcache' supervisor diagnostic shell command.

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
