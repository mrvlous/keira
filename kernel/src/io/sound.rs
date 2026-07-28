// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(dead_code)]
//! Keira Kernel: PC Speaker Sound Driver (Rust FFI Wrapper)
//!
//! Safe Rust interface to the C PC Speaker driver (`drivers/sound/`).
//!
//! These functions call into C via FFI. The C driver handles the actual
//! hardware interaction (PIT Channel 2 programming and speaker gate control
//! via `outb`/`inb` instructions on ports 0x42, 0x43, and 0x61).
//!
//! Why wrap C functions instead of reimplementing in Rust?
//!   - Keeps hardware I/O port access centralized in the C driver layer.
//!   - Avoids duplicating inline assembly for `outb`/`inb`.
//!   - Demonstrates the C↔Rust interop that is central to Keira's design.

use core::arch::asm;

// FFI Declarations : C functions from `drivers/sound/sound.c`
extern "C" {
    /// Play a tone at the specified frequency on the PC Speaker.
    fn sound_play(freq: u32);

    /// Stop all sound output on the PC Speaker.
    fn sound_stop();

    /// Get the system uptime in milliseconds (from PIT tick counter).
    fn get_uptime_ms() -> u64;
}

// Safe Public API

/// Play a tone at the specified frequency (in Hz) on the PC Speaker.
///
/// Configures PIT Channel 2 as a square wave generator and enables the
/// speaker gate via System Control Port B.
///
/// # Arguments
/// * `freq` : The target frequency in Hz. A value of 0 is silently ignored.
pub fn play_sound(freq: u32) {
    // SAFETY: `sound_play` is a simple I/O operation with no memory
    // side effects beyond writing to PIT and speaker gate registers.
    unsafe {
        sound_play(freq);
    }
}

/// Stop all sound output on the PC Speaker.
///
/// Clears the speaker gate and PIT Channel 2 gate bits in
/// System Control Port B (0x61).
pub fn stop_sound() {
    // SAFETY: `sound_stop` is a simple I/O operation that clears
    // speaker gate bits with no memory side effects.
    unsafe {
        sound_stop();
    }
}

/// Busy-wait sleep helper using CPU `hlt` instructions.
///
/// Suspends execution for the specified duration by polling the kernel
/// uptime counter and halting the CPU between checks.
///
/// # Arguments
/// * `ms` : The number of milliseconds to sleep.
pub fn sleep_ms(ms: u64) {
    let start = unsafe { get_uptime_ms() };
    while unsafe { get_uptime_ms() } < start + ms {
        unsafe {
            asm!("hlt");
        }
    }
}

/// Play a musical note for a specified duration followed by a short gap.
///
/// If the frequency is 0, the function produces silence (rest) for the
/// given duration. A 10ms gap is inserted after each note to distinguish
/// consecutive identical pitches.
///
/// # Arguments
/// * `freq`        : The note frequency in Hz (0 for silence/rest).
/// * `duration_ms` : The duration of the note in milliseconds.
pub fn play_note(freq: u32, duration_ms: u64) {
    if freq == 0 {
        stop_sound();
        sleep_ms(duration_ms);
    } else {
        play_sound(freq);
        sleep_ms(duration_ms);
        stop_sound();
    }
    // 10ms gap between notes for articulation
    sleep_ms(10);
}
