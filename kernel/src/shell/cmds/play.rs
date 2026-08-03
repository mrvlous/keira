#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'play'
//!
//! Plays predefined retro melodies or beeps on the PC speaker.

use crate::io::sound;

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let melody = match parts.next() {
        Some("-h") | Some("--help") => {
            vga::print_str("Usage: play <mario|nokia|starwars|beep>\n\n");
            vga::print_str("Description:\n  Play predefined retro chiptune melodies or simple audio beeps on the PC speaker.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  play mario\n  play beep\n");
            return;
        }
        Some(s) => s,
        None => {
            vga::print_str("Usage: play <mario|nokia|starwars|beep>\n");
            return;
        }
    };

    match melody {
        "beep" => {
            vga::print_str("Playing simple beep...\n");
            sound::play_note(1000, 200);
        }
        "mario" => {
            vga::print_str("Playing Mario Theme...\n");
            let notes = [
                (660, 100),
                (660, 100),
                (0, 100),
                (660, 100),
                (0, 100),
                (510, 100),
                (660, 100),
                (0, 100),
                (770, 100),
                (0, 300),
                (380, 100),
            ];
            for &(freq, dur) in notes.iter() {
                sound::play_note(freq, dur);
            }
        }
        "nokia" => {
            vga::print_str("Playing Nokia Tune...\n");
            let notes = [
                (659, 150),
                (587, 150),
                (370, 300),
                (415, 300),
                (554, 150),
                (494, 150),
                (294, 300),
                (330, 300),
                (494, 150),
                (440, 150),
                (277, 300),
                (330, 300),
                (440, 500),
            ];
            for &(freq, dur) in notes.iter() {
                sound::play_note(freq, dur);
            }
        }
        "starwars" => {
            vga::print_str("Playing Star Wars Theme...\n");
            let notes = [
                (392, 300),
                (392, 300),
                (392, 300),
                (523, 600),
                (784, 600),
                (698, 150),
                (659, 150),
                (587, 150),
                (1046, 600),
                (784, 300),
                (698, 150),
                (659, 150),
                (587, 150),
                (1046, 600),
                (784, 300),
                (698, 150),
                (659, 150),
                (698, 150),
                (587, 600),
            ];
            for &(freq, dur) in notes.iter() {
                sound::play_note(freq, dur);
            }
        }
        _ => {
            vga::print_str("Unknown melody. Try: mario, nokia, starwars, beep\n");
        }
    }
}
