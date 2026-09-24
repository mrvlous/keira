// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Keyboard Controller hardware driver, scancode decoding, and event dispatch.

use super::scancode::{
    KBD_US_LAYOUT, KBD_US_SHIFTED_LAYOUT, KEY_DOWN, KEY_F10, KEY_F3, KEY_LCTRL, KEY_LEFT,
    KEY_LSHIFT, KEY_RIGHT, KEY_RSHIFT, KEY_UP,
};
use core::sync::atomic::{AtomicBool, Ordering};
use keira_arch::cpu::inb;
use keira_arch::interrupts::pic;

/// I/O port address for PS/2 keyboard data.
pub const KBD_DATA_PORT: u16 = 0x60;

static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static EXTENDED_SCANCODE: AtomicBool = AtomicBool::new(false);

#[cfg(not(test))]
extern "C" {
    fn shell_handle_keypress(c: u8);
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn shell_handle_keypress(_c: u8) {}

/// Capacity in bytes for the circular keyboard scancode/ASCII input ring buffer.
pub const KBD_QUEUE_SIZE: usize = 128;
static mut KBD_QUEUE: [u8; KBD_QUEUE_SIZE] = [0u8; KBD_QUEUE_SIZE];
static mut KBD_HEAD: usize = 0;
static mut KBD_TAIL: usize = 0;

/// Initializes keyboard hardware and unmasks IRQ1 on Dual 8259 PIC.
pub fn init() {
    pic::clear_mask(1);
}

/// IRQ1 interrupt service routine handling PS/2 keyboard scancodes.
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
                    push_input_char(c);
                    let input_ch = if CTRL_PRESSED.load(Ordering::Relaxed) {
                        if (b'a'..=b'z').contains(&c) {
                            c - b'a' + 1
                        } else if (b'A'..=b'Z').contains(&c) {
                            c - b'A' + 1
                        } else {
                            c
                        }
                    } else {
                        c
                    };
                    crate::tty::push_char(input_ch);
                    shell_handle_keypress(input_ch);
                }
            }
        }

        pic::send_eoi(1);
    }
}

/// Enqueues an ASCII character into the keyboard device input queue.
///
/// # Safety
///
/// Mutates global keyboard ring buffer indices.
pub unsafe fn push_input_char(c: u8) {
    let next_head = (KBD_HEAD + 1) % KBD_QUEUE_SIZE;
    if next_head != KBD_TAIL {
        KBD_QUEUE[KBD_HEAD] = c;
        KBD_HEAD = next_head;
    }
}

/// Dequeues an ASCII character from the keyboard device input queue.
///
/// # Safety
///
/// Mutates global keyboard ring buffer indices.
pub unsafe fn pop_input_char() -> Option<u8> {
    if KBD_HEAD == KBD_TAIL {
        None
    } else {
        let c = KBD_QUEUE[KBD_TAIL];
        KBD_TAIL = (KBD_TAIL + 1) % KBD_QUEUE_SIZE;
        Some(c)
    }
}

/// Returns the number of characters currently enqueued in the keyboard device input queue.
///
/// # Safety
///
/// Reads global keyboard ring buffer indices.
pub unsafe fn input_queue_len() -> usize {
    if KBD_HEAD >= KBD_TAIL {
        KBD_HEAD - KBD_TAIL
    } else {
        KBD_QUEUE_SIZE - (KBD_TAIL - KBD_HEAD)
    }
}
