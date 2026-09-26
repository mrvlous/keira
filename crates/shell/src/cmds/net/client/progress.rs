// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Terminal transfer progress indicators, byte metric formatters, and status badges.

use keira_io::vga;

/// Prints standard 12-character right-aligned status badge in bracketed Cargo style.
///
/// # Safety
/// Writes directly to the VGA text mode console.
pub unsafe fn print_status_badge(tag: &str, color: vga::Color) {
    vga::set_color(color, vga::Color::Black);
    for _ in 0..(12usize.saturating_sub(tag.len())) {
        vga::print_str(" ");
    }
    vga::print_str(tag);
    vga::print_str(" ");
    vga::set_color(vga::Color::White, vga::Color::Black);
}

/// Print byte quantity in human-readable units (Bytes, KiB, MiB).
///
/// # Safety
/// Writes to the VGA console.
pub unsafe fn print_byte_size(bytes: usize) {
    if bytes < 1024 {
        vga::print_u64(bytes as u64);
        vga::print_str(" B");
    } else if bytes < 1024 * 1024 {
        vga::print_u64((bytes / 1024) as u64);
        vga::print_str(".");
        vga::print_u64(((bytes % 1024) * 10 / 1024) as u64);
        vga::print_str(" KiB");
    } else {
        vga::print_u64((bytes / (1024 * 1024)) as u64);
        vga::print_str(".");
        vga::print_u64(((bytes % (1024 * 1024)) * 10 / (1024 * 1024)) as u64);
        vga::print_str(" MiB");
    }
}

/// Dynamic ASCII progress tracker.
pub struct ProgressRenderer {
    pub bar_width: usize,
    pub last_percent: usize,
}

impl ProgressRenderer {
    /// Create a new progress renderer with the specified bar character width.
    pub const fn new(bar_width: usize) -> Self {
        Self {
            bar_width,
            last_percent: 0,
        }
    }

    /// Render standard transfer progress line.
    ///
    /// # Safety
    /// Writes directly to VGA buffer.
    pub unsafe fn update(&mut self, received: usize, total_opt: Option<usize>) {
        if let Some(total) = total_opt {
            if total > 0 {
                let percent = (received * 100) / total;
                print_status_badge("Downloading", vga::Color::LightGreen);
                vga::print_str("[");
                let filled = (percent * self.bar_width) / 100;
                for _ in 0..filled {
                    vga::print_str("=");
                }
                if filled < self.bar_width {
                    vga::print_str(">");
                    for _ in (filled + 1)..self.bar_width {
                        vga::print_str(" ");
                    }
                }
                vga::print_str("] ");
                vga::print_u64(percent as u64);
                vga::print_str("% (");
                print_byte_size(received);
                vga::print_str(" / ");
                print_byte_size(total);
                vga::print_str(")\r");
                self.last_percent = percent;
                return;
            }
        }

        print_status_badge("Downloading", vga::Color::LightGreen);
        vga::print_str("Streamed: ");
        print_byte_size(received);
        vga::print_str("\r");
    }

    /// Render final completion badge.
    ///
    /// # Safety
    /// Writes to VGA console.
    pub unsafe fn finish(&self, received: usize) {
        vga::print_str("\n");
        print_status_badge("Finished", vga::Color::LightGreen);
        vga::print_str("Completed ");
        print_byte_size(received);
        vga::print_str(" transfer\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_renderer_initialization() {
        let renderer = ProgressRenderer::new(28);
        assert_eq!(renderer.bar_width, 28);
        assert_eq!(renderer.last_percent, 0);
    }
}
