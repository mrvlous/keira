// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'rnginfo' and 'entropy' shell commands.

use crate::args::CliArgs;
use keira_arch::cpu::get_entropy_features;
use keira_io::vga;

/// Formats a single byte as a two-digit hexadecimal string.
fn print_hex_byte(b: u8) {
    let hi = b >> 4;
    let lo = b & 0xF;
    let ch_hi = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
    let ch_lo = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
    let buf = [ch_hi, ch_lo];
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

/// Formats a boolean feature status as standard badge tokens.
fn print_support_badge(supported: bool) {
    if supported {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[SUPPORTED]  ");
    } else {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("[UNSUPPORTED]");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

/// Executes the 'rnginfo' / 'entropy' diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: rnginfo [-v] [-h]\n\n");
        vga::print_str(
            "Description:\n  Display hardware entropy harvesting and cryptographic random engine status.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str(
            "  -v, --verbose  Display live silicon entropy sampling and hardware feature flags\n",
        );
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let entropy_feat = get_entropy_features();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Silicon Hardware Entropy & Random Number Generator Subsystem:\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Hardware Source:   ");
    if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[HARDWARE]    ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" CPU RDRAND");
        if entropy_feat.has_rdseed {
            vga::print_str(" + RDSEED");
        }
        vga::print_str("\n");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[FALLBACK]    ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" CPU TSC/HPET SplitMix64 Mixer\n");
    }

    vga::print_str("  Entropy Quality:   ");
    if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[HIGH]        ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" Hardware Quantum Thermal Noise\n");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::print_str("[STANDARD]    ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" High-Resolution Cycle Jitter\n");
    }

    vga::print_str("  Kernel Subsystems:\n");
    vga::print_str("    AT_RANDOM:       ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[ACTIVE]      ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" 16-byte silicon seed per task execution stack\n");

    vga::print_str("    Character Nodes: ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[ACTIVE]      ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" /dev/random and /dev/urandom\n");

    vga::print_str("    Syscall Vector:  ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[ACTIVE]      ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(" Syscall 101 SYS_GETRANDOM (Linux 318)\n");

    if args.has_flag('v', "verbose") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("\n  Live Silicon Entropy Harvest Sample:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("    Sample (16 B):   0x");

        for _ in 0..4 {
            let val = keira_arch::cpu::get_entropy_u32();
            let bytes = val.to_be_bytes();
            for b in bytes {
                print_hex_byte(b);
            }
        }
        vga::print_str(" (harvested)\n\n");

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("  CPU Silicon Features:\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        vga::print_str("    RDRAND:          ");
        print_support_badge(entropy_feat.has_rdrand);
        vga::print_str(" Hardware digital random number generator\n");

        vga::print_str("    RDSEED:          ");
        print_support_badge(entropy_feat.has_rdseed);
        vga::print_str(" Direct physical noise source generator\n");

        vga::print_str("    RDTSC:           ");
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[SUPPORTED]  ");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str(" High-resolution timestamp counter\n");
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
