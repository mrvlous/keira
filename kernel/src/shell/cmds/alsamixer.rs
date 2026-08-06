#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'alsamixer'
//!
//! Implementation of the 'alsamixer' shell command to query and adjust master volume
//! gain and mute state on Intel HDA DSP audio controller (Syscall 71).

use crate::io::{audio, vga};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let arg = parts.next();
    if arg == Some("-h") || arg == Some("--help") {
        unsafe {
            vga::print_str("Usage: alsamixer [volume_percent 0..100]\n\n");
            vga::print_str("Description:\n  Query or adjust Intel HDA DSP master volume gain and mute state (Syscall 71).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        if let Some(vol_str) = arg {
            if let Ok(vol) = vol_str.parse::<u8>() {
                let clamped = if vol > 100 { 100 } else { vol };
                let _ = audio::sys_audio_dsp(audio::AUDIO_CMD_VOLUME, clamped as u64, 0);
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Intel HDA Master Volume Gain set to ");
                vga::print_u64(clamped as u64);
                vga::print_str("%\n");
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("alsamixer: Invalid volume percentage. Specify 0..100.\n");
            }
        } else {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("┌──────────────────────────────────────────────────────────┐\n");
            vga::print_str("│ Keira Kernel Intel HDA Audio DSP Control Panel           │\n");
            vga::print_str("├──────────────────────────────────────────────────────────┤\n");
            vga::print_str("│ Active Controller : Intel HD Audio (Vendor 0x8086)       │\n");
            vga::print_str("│ Master Volume     : [");
            let vol = audio::MASTER_VOLUME;
            let filled = (vol / 5) as usize;
            for i in 0..20 {
                if i < filled {
                    vga::print_str("#");
                } else {
                    vga::print_str("-");
                }
            }
            vga::print_str("] ");
            vga::print_u64(vol as u64);
            vga::print_str("%               │\n");
            vga::print_str("│ Output Channel    : Speaker / Headphone Jack (Stereo)    │\n");
            vga::print_str("└──────────────────────────────────────────────────────────┘\n");
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
