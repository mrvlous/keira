// SPDX-License-Identifier: GPL-2.0-only

//! Flush dirty filesystem block cache pages to physical storage device.

use keira_fs::fat;
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: sync [-h]\n\n");
        vga::print_str(
            "Description:\n  Flush dirty filesystem block cache pages to physical storage.\n\n",
        );
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    unsafe {
        let dirty_inodes = keira_fs::cache::icache_flush_dirty();
        if dirty_inodes > 0 {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("sync: flushed ");
            vga::print_u64(dirty_inodes as u64);
            vga::print_str(" dirty inodes to storage\n");
        }

        match fat::flush_dirty_sectors() {
            Ok(count) => {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("sync: flushed ");
                vga::print_u64(count as u64);
                vga::print_str(" dirty sectors to storage\n");
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("sync: error: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
