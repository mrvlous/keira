// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! PS/2 Keyboard driver and scancode translation.

pub mod driver {
    use super::scancode::{
        KBD_US_LAYOUT, KBD_US_SHIFTED_LAYOUT, KEY_DOWN, KEY_F10, KEY_F3, KEY_LCTRL, KEY_LEFT,
        KEY_LSHIFT, KEY_RIGHT, KEY_RSHIFT, KEY_UP,
    };
    use core::sync::atomic::{AtomicBool, Ordering};
    use keira_arch::cpu::inb;
    use keira_arch::interrupts::pic;

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

    pub const KBD_QUEUE_SIZE: usize = 128;
    static mut KBD_QUEUE: [u8; KBD_QUEUE_SIZE] = [0u8; KBD_QUEUE_SIZE];
    static mut KBD_HEAD: usize = 0;
    static mut KBD_TAIL: usize = 0;

    pub fn init() {
        pic::clear_mask(1);
    }

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

                let is_raw_tty = (crate::tty::get_termios().c_lflag & crate::tty::ICANON) == 0;
                if scancode == KEY_UP {
                    if is_raw_tty {
                        crate::tty::push_char(0x1b);
                        crate::tty::push_char(b'[');
                        crate::tty::push_char(b'A');
                    }
                    shell_handle_keypress(0x80);
                } else if scancode == KEY_DOWN {
                    if is_raw_tty {
                        crate::tty::push_char(0x1b);
                        crate::tty::push_char(b'[');
                        crate::tty::push_char(b'B');
                    }
                    shell_handle_keypress(0x81);
                } else if scancode == KEY_LEFT {
                    if is_raw_tty {
                        crate::tty::push_char(0x1b);
                        crate::tty::push_char(b'[');
                        crate::tty::push_char(b'D');
                    }
                    shell_handle_keypress(0x82);
                } else if scancode == KEY_RIGHT {
                    if is_raw_tty {
                        crate::tty::push_char(0x1b);
                        crate::tty::push_char(b'[');
                        crate::tty::push_char(b'C');
                    }
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
                            if c.is_ascii_lowercase() {
                                c - b'a' + 1
                            } else if c.is_ascii_uppercase() {
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

    pub unsafe fn push_input_char(c: u8) {
        let next_head = (KBD_HEAD + 1) % KBD_QUEUE_SIZE;
        if next_head != KBD_TAIL {
            KBD_QUEUE[KBD_HEAD] = c;
            KBD_HEAD = next_head;
        }
    }

    pub unsafe fn pop_input_char() -> Option<u8> {
        if KBD_HEAD == KBD_TAIL {
            None
        } else {
            let c = KBD_QUEUE[KBD_TAIL];
            KBD_TAIL = (KBD_TAIL + 1) % KBD_QUEUE_SIZE;
            Some(c)
        }
    }

    pub unsafe fn input_queue_len() -> usize {
        if KBD_HEAD >= KBD_TAIL {
            KBD_HEAD - KBD_TAIL
        } else {
            KBD_QUEUE_SIZE - (KBD_TAIL - KBD_HEAD)
        }
    }

    pub unsafe fn flush_input_queue() {
        KBD_HEAD = 0;
        KBD_TAIL = 0;
    }
}

pub mod scancode {
    pub const KEY_LSHIFT: u8 = 0x2A;

    pub const KEY_RSHIFT: u8 = 0x36;

    pub const KEY_LCTRL: u8 = 0x1D;

    pub const KEY_UP: u8 = 0x48;

    pub const KEY_DOWN: u8 = 0x50;

    pub const KEY_LEFT: u8 = 0x4B;

    pub const KEY_RIGHT: u8 = 0x4D;

    pub const KEY_F3: u8 = 0x3D;

    pub const KEY_F10: u8 = 0x44;

    pub const KBD_US_LAYOUT: [u8; 128] = {
        let mut table = [0u8; 128];
        table[0x01] = 27;
        table[0x02] = b'1';
        table[0x03] = b'2';
        table[0x04] = b'3';
        table[0x05] = b'4';
        table[0x06] = b'5';
        table[0x07] = b'6';
        table[0x08] = b'7';
        table[0x09] = b'8';
        table[0x0A] = b'9';
        table[0x0B] = b'0';
        table[0x0C] = b'-';
        table[0x0D] = b'=';
        table[0x0E] = b'\x08';
        table[0x0F] = b'\t';
        table[0x10] = b'q';
        table[0x11] = b'w';
        table[0x12] = b'e';
        table[0x13] = b'r';
        table[0x14] = b't';
        table[0x15] = b'y';
        table[0x16] = b'u';
        table[0x17] = b'i';
        table[0x18] = b'o';
        table[0x19] = b'p';
        table[0x1A] = b'[';
        table[0x1B] = b']';
        table[0x1C] = b'\n';
        table[0x1E] = b'a';
        table[0x1F] = b's';
        table[0x20] = b'd';
        table[0x21] = b'f';
        table[0x22] = b'g';
        table[0x23] = b'h';
        table[0x24] = b'j';
        table[0x25] = b'k';
        table[0x26] = b'l';
        table[0x27] = b';';
        table[0x28] = b'\'';
        table[0x29] = b'`';
        table[0x2B] = b'\\';
        table[0x2C] = b'z';
        table[0x2D] = b'x';
        table[0x2E] = b'c';
        table[0x2F] = b'v';
        table[0x30] = b'b';
        table[0x31] = b'n';
        table[0x32] = b'm';
        table[0x33] = b',';
        table[0x34] = b'.';
        table[0x35] = b'/';
        table[0x37] = b'*';
        table[0x39] = b' ';
        table
    };

    pub const KBD_US_SHIFTED_LAYOUT: [u8; 128] = {
        let mut table = [0u8; 128];
        table[0x01] = 27;
        table[0x02] = b'!';
        table[0x03] = b'@';
        table[0x04] = b'#';
        table[0x05] = b'$';
        table[0x06] = b'%';
        table[0x07] = b'^';
        table[0x08] = b'&';
        table[0x09] = b'*';
        table[0x0A] = b'(';
        table[0x0B] = b')';
        table[0x0C] = b'_';
        table[0x0D] = b'+';
        table[0x0E] = b'\x08';
        table[0x0F] = b'\t';
        table[0x10] = b'Q';
        table[0x11] = b'W';
        table[0x12] = b'E';
        table[0x13] = b'R';
        table[0x14] = b'T';
        table[0x15] = b'Y';
        table[0x16] = b'U';
        table[0x17] = b'I';
        table[0x18] = b'O';
        table[0x19] = b'P';
        table[0x1A] = b'{';
        table[0x1B] = b'}';
        table[0x1C] = b'\n';
        table[0x1E] = b'A';
        table[0x1F] = b'S';
        table[0x20] = b'D';
        table[0x21] = b'F';
        table[0x22] = b'G';
        table[0x23] = b'H';
        table[0x24] = b'J';
        table[0x25] = b'K';
        table[0x26] = b'L';
        table[0x27] = b':';
        table[0x28] = b'"';
        table[0x29] = b'~';
        table[0x2B] = b'|';
        table[0x2C] = b'Z';
        table[0x2D] = b'X';
        table[0x2E] = b'C';
        table[0x2F] = b'V';
        table[0x30] = b'B';
        table[0x31] = b'N';
        table[0x32] = b'M';
        table[0x33] = b'<';
        table[0x34] = b'>';
        table[0x35] = b'?';
        table[0x37] = b'*';
        table[0x39] = b' ';
        table
    };
}

pub use self::driver::*;
pub use self::scancode::*;

#[cfg(test)]
mod tests {
    use super::driver::{input_queue_len, pop_input_char, push_input_char};
    use super::scancode::{KBD_US_LAYOUT, KBD_US_SHIFTED_LAYOUT};

    #[test]
    fn test_scancode_ascii_mapping() {
        // 0x1E is 'a'
        assert_eq!(KBD_US_LAYOUT[0x1E], b'a');
        assert_eq!(KBD_US_SHIFTED_LAYOUT[0x1E], b'A');

        // 0x10 is 'q'
        assert_eq!(KBD_US_LAYOUT[0x10], b'q');
        assert_eq!(KBD_US_SHIFTED_LAYOUT[0x10], b'Q');

        // 0x02 is '1' / '!'
        assert_eq!(KBD_US_LAYOUT[0x02], b'1');
        assert_eq!(KBD_US_SHIFTED_LAYOUT[0x02], b'!');

        // 0x1C is Enter ('\n')
        assert_eq!(KBD_US_LAYOUT[0x1C], b'\n');
        assert_eq!(KBD_US_SHIFTED_LAYOUT[0x1C], b'\n');
    }

    #[test]
    fn test_keyboard_input_queue() {
        unsafe {
            // Drain any preexisting elements
            while pop_input_char().is_some() {}
            assert_eq!(input_queue_len(), 0);

            push_input_char(b'H');
            push_input_char(b'e');
            push_input_char(b'l');
            push_input_char(b'l');
            push_input_char(b'o');

            assert_eq!(input_queue_len(), 5);
            assert_eq!(pop_input_char(), Some(b'H'));
            assert_eq!(pop_input_char(), Some(b'e'));
            assert_eq!(pop_input_char(), Some(b'l'));
            assert_eq!(pop_input_char(), Some(b'l'));
            assert_eq!(pop_input_char(), Some(b'o'));
            assert_eq!(pop_input_char(), None);
            assert_eq!(input_queue_len(), 0);
        }
    }
}
