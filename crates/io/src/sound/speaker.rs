// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pure Rust PC Speaker PIT Channel 2 sound synthesizer and sleep timers.

use core::arch::asm;
use keira_arch::cpu::{inb, outb};
use keira_arch::timers::pit::uptime_ms;

const PIT_BASE_FREQ: u32 = 1193182;
const PIT_CHANNEL2_DATA: u16 = 0x42;
const PIT_COMMAND_PORT: u16 = 0x43;
const PC_SPEAKER_PORT: u16 = 0x61;

/// Play a tone at specified frequency (in Hz) on the PC Speaker in pure Rust.
pub fn play_sound(freq: u32) {
    if freq == 0 {
        stop_sound();
        return;
    }

    let divisor = (PIT_BASE_FREQ / freq.max(1)).min(65535) as u16;

    unsafe {
        // Set PIT Channel 2 to Mode 3 (Square wave generator)
        outb(PIT_COMMAND_PORT, 0xB6);
        outb(PIT_CHANNEL2_DATA, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL2_DATA, ((divisor >> 8) & 0xFF) as u8);

        // Enable PC Speaker gate and output bits (bits 0 and 1)
        let state = inb(PC_SPEAKER_PORT);
        if (state & 0x03) != 0x03 {
            outb(PC_SPEAKER_PORT, state | 0x03);
        }
    }
}

/// Stop all sound output on the PC Speaker.
pub fn stop_sound() {
    unsafe {
        let state = inb(PC_SPEAKER_PORT);
        outb(PC_SPEAKER_PORT, state & 0xFC);
    }
}

// C-compatible exports
#[no_mangle]
pub extern "C" fn sound_play(freq: u32) {
    play_sound(freq);
}

#[no_mangle]
pub extern "C" fn sound_stop() {
    stop_sound();
}

/// Sleep for specified duration in milliseconds using pure Rust PIT uptime.
pub fn sleep_ms(ms: u64) {
    let start = uptime_ms();
    while uptime_ms() < start + ms {
        unsafe {
            asm!("hlt");
        }
    }
}

/// Play musical note tone followed by short articulation gap.
pub fn play_note(freq: u32, duration_ms: u64) {
    if freq == 0 {
        stop_sound();
        sleep_ms(duration_ms);
    } else {
        play_sound(freq);
        sleep_ms(duration_ms);
        stop_sound();
    }
    sleep_ms(10);
}
