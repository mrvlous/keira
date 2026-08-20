// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PC Speaker PIT Channel 2 sound synthesizer and sleep timers.

use core::arch::asm;

extern "C" {
    fn sound_play(freq: u32);
    fn sound_stop();
    fn get_uptime_ms() -> u64;
}

/// Play a tone at specified frequency (in Hz) on the PC Speaker.
pub fn play_sound(freq: u32) {
    unsafe {
        sound_play(freq);
    }
}

/// Stop all sound output on the PC Speaker.
pub fn stop_sound() {
    unsafe {
        sound_stop();
    }
}

/// Sleep for specified duration in milliseconds.
pub fn sleep_ms(ms: u64) {
    let start = unsafe { get_uptime_ms() };
    while unsafe { get_uptime_ms() } < start + ms {
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
