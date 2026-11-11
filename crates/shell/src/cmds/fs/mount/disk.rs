// SPDX-License-Identifier: GPL-2.0-only

//! Implementation of the 'disk' shell command.

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
