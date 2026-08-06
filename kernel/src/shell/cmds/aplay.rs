#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'aplay'
//!
//! Implementation of the 'aplay' shell command to parse and stream WAV audio payloads
//! from FAT16 disk storage through Intel HDA DSP DMA engine (Syscall 71).

use crate::io::{audio, vga};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let filename = match parts.next() {
        Some("-h") | Some("--help") => {
            unsafe {
                vga::print_str("Usage: aplay <file.wav>\n\n");
                vga::print_str("Description:\n  Parse and stream RIFF WAV PCM audio payload from FAT16 storage through Intel HDA DSP DMA engine (Syscall 71).\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            }
            return;
        }
        Some(f) => f,
        None => {
            unsafe {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: aplay <file.wav>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
    };

    unsafe {
        let _ = audio::sys_audio_dsp(audio::AUDIO_CMD_PLAY, 0, 0);

        let mut file_buf = [0u8; 8192];
        match crate::fs::fat::read_file_content(filename, &mut file_buf) {
            Ok(len) => {
                let payload = &file_buf[..len];
                match audio::parse_wav_header(payload) {
                    Ok(header) => {
                        let pcm_data = &payload[44..];
                        if let Err(err) = audio::stream_pcm_buffer(&header, pcm_data) {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("aplay error: ");
                            vga::print_str(err);
                            vga::print_str("\n");
                        }
                    }
                    Err(err) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("aplay error: ");
                        vga::print_str(err);
                        vga::print_str("\n");
                    }
                }
            }
            Err(_) => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("aplay: ");
                vga::print_str(filename);
                vga::print_str(" not found on FAT16 disk. Simulating PCM tone stream...\n");

                let dummy_hdr = audio::WavHeader {
                    riff_id: *b"RIFF",
                    chunk_size: 352844,
                    wave_id: *b"WAVE",
                    fmt_id: *b"fmt ",
                    fmt_chunk_size: 16,
                    audio_format: 1,
                    num_channels: 2,
                    sample_rate: 44100,
                    byte_rate: 176400,
                    block_align: 4,
                    bits_per_sample: 16,
                    data_id: *b"data",
                    data_size: 352800,
                };
                let dummy_pcm = [0x80u8; 1024];
                let _ = audio::stream_pcm_buffer(&dummy_hdr, &dummy_pcm);
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
