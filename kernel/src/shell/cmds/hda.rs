#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'hda'
//!
//! Plays sound using the Intel High Definition Audio (HDA) controller.

use crate::io::hda;

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let action = match parts.next() {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: hda <status|play [freq]|stop>\n\n");
            vga::print_str("Description:\n  Query status or generate continuous audio waveforms using the Intel High Definition Audio (HDA) DMA controller.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Subcommands:\n  status       Query Intel HDA controller PCI mapping state\n  play [freq]  Play continuous tone at frequency (1-20000 Hz, default 440 Hz)\n  stop         Silence active HDA audio stream\n");
            return;
        }
        Some(s) => s,
        None => {
            vga::print_str("Usage: hda <status|play [freq]|stop>\n");
            return;
        }
    };

    match action {
        "status" => unsafe {
            if hda::HDA_INITIALIZED {
                vga::print_str("Intel HD Audio Controller: Mapped & Initialized (Active)\n");
            } else if hda::HDA_PCI_FOUND {
                vga::print_str(
                    "Intel HD Audio Controller: Found on PCI but failed to initialize\n",
                );
            } else {
                vga::print_str("Intel HD Audio Controller: Not detected on PCI bus\n");
            }
        },
        "play" => {
            unsafe {
                if !hda::HDA_INITIALIZED {
                    vga::print_str("Error: HDA is not initialized.\n");
                    return;
                }
            }
            let freq = match parts.next() {
                Some(s) => match s.parse::<u32>() {
                    Ok(val) => val,
                    Err(_) => {
                        vga::print_str("Error: Invalid frequency value.\n");
                        return;
                    }
                },
                // Default A4 note
                None => 440,
            };
            if freq == 0 || freq > 20000 {
                vga::print_str("Error: Frequency must be between 1 and 20000 Hz.\n");
                return;
            }
            vga::print_str("Starting continuous HDA tone at ");
            vga::print_u64(freq as u64);
            vga::print_str(" Hz...\n");
            hda::play_tone(freq);
        }
        "stop" => {
            unsafe {
                if !hda::HDA_INITIALIZED {
                    vga::print_str("Error: HDA is not initialized.\n");
                    return;
                }
            }
            vga::print_str("Stopping HDA audio stream.\n");
            hda::stop();
        }
        _ => {
            vga::print_str("Unknown action. Try: hda play <freq>, hda stop, hda status\n");
        }
    }
}
