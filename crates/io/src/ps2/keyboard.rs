// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Keyboard Controller Driver in pure Rust.

use core::sync::atomic::{AtomicBool, Ordering};
use keira_arch::cpu::inb;
use keira_arch::interrupts::pic;

pub const KBD_DATA_PORT: u16 = 0x60;

const KEY_LSHIFT: u8 = 0x2A;
const KEY_RSHIFT: u8 = 0x36;
const KEY_LCTRL: u8 = 0x1D;
const KEY_UP: u8 = 0x48;
const KEY_DOWN: u8 = 0x50;
const KEY_LEFT: u8 = 0x4B;
const KEY_RIGHT: u8 = 0x4D;
const KEY_F3: u8 = 0x3D;
const KEY_F10: u8 = 0x44;

static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static EXTENDED_SCANCODE: AtomicBool = AtomicBool::new(false);

const KBD_US_LAYOUT: [u8; 128] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\x08', b'\t',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n', 0, b'a', b's',
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\', b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0, b' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KBD_US_SHIFTED_LAYOUT: [u8; 128] = [
    0, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', b'\x08', b'\t',
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', b'\n', 0, b'A', b'S',
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', 0, b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0, b'*', 0, b' ', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

extern "Rust" {
    fn shell_handle_keypress(c: u8);
}

/// Initialize and unmask keyboard IRQ1 on Dual 8259 PIC.
pub fn init() {
    pic::clear_mask(1);
}

/// Read scancodes from keyboard data port 0x60 and dispatch keypress events.
#[no_mangle]
pub extern "C" fn keyboard_handler() {
    unsafe {
        let scancode = inb(KBD_DATA_PORT);

        if scancode == 0xE0 {
            EXTENDED_SCANCODE.store(true, Ordering::Relaxed);
            pic::send_eoi(1);
            return;
        }

        let is_extended = EXTENDED_SCANCODE.swap(false, Ordering::Relaxed);

        if (scancode & 0x80) != 0 {
            // Key release event
            if !is_extended {
                if scancode == (KEY_LSHIFT | 0x80) || scancode == (KEY_RSHIFT | 0x80) {
                    SHIFT_PRESSED.store(false, Ordering::Relaxed);
                } else if scancode == (KEY_LCTRL | 0x80) {
                    CTRL_PRESSED.store(false, Ordering::Relaxed);
                }
            }
        } else {
            // Key press event
            if !is_extended {
                if scancode == KEY_LSHIFT || scancode == KEY_RSHIFT {
                    SHIFT_PRESSED.store(true, Ordering::Relaxed);
                } else if scancode == KEY_LCTRL {
                    CTRL_PRESSED.store(true, Ordering::Relaxed);
                }
            }

            if scancode == KEY_UP {
                shell_handle_keypress(0x80);
            } else if scancode == KEY_DOWN {
                shell_handle_keypress(0x81);
            } else if scancode == KEY_LEFT {
                shell_handle_keypress(0x82);
            } else if scancode == KEY_RIGHT {
                shell_handle_keypress(0x83);
            } else if scancode == KEY_F3 {
                shell_handle_keypress(0x84);
            } else if scancode == KEY_F10 {
                shell_handle_keypress(0x85);
            } else if (scancode as usize) < 128 {
                let shifted = SHIFT_PRESSED.load(Ordering::Relaxed);
                let c = if shifted {
                    KBD_US_SHIFTED_LAYOUT[scancode as usize]
                } else {
                    KBD_US_LAYOUT[scancode as usize]
                };

                if c != 0 {
                    if CTRL_PRESSED.load(Ordering::Relaxed) {
                        if c >= b'a' && c <= b'z' {
                            shell_handle_keypress(c - b'a' + 1);
                            pic::send_eoi(1);
                            return;
                        }
                        if c >= b'A' && c <= b'Z' {
                            shell_handle_keypress(c - b'A' + 1);
                            pic::send_eoi(1);
                            return;
                        }
                    }
                    shell_handle_keypress(c);
                }
            }
        }

        pic::send_eoi(1);
    }
}
