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

//! N_TTY line discipline, canonical input editing and termios flags.

pub mod discipline {
    use super::termios::{Termios, ICANON, ISIG, VEOF, VERASE, VINTR};

    static mut ACTIVE_TERMIOS: Termios = Termios {
        c_iflag: 0x0500,
        c_oflag: 0x0005,
        c_cflag: 0x00BF,
        c_lflag: 0x8A3B,
        c_line: 0,
        c_cc: [
            3, 28, 127, 21, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
        c_ispeed: 38400,
        c_ospeed: 38400,
    };

    pub const COOKED_BUF_SIZE: usize = 512;
    static mut COOKED_QUEUE: [u8; COOKED_BUF_SIZE] = [0; COOKED_BUF_SIZE];
    static mut COOKED_HEAD: usize = 0;
    static mut COOKED_TAIL: usize = 0;

    pub const LINE_BUF_SIZE: usize = 256;
    static mut LINE_BUF: [u8; LINE_BUF_SIZE] = [0; LINE_BUF_SIZE];
    static mut LINE_LEN: usize = 0;
    static mut LAST_CHAR_WAS_CR: bool = false;
    static mut LAST_BACKSPACE_POPPED: bool = false;
    static mut EOF_PENDING: bool = false;

    pub fn take_eof_pending() -> bool {
        unsafe {
            if EOF_PENDING {
                EOF_PENDING = false;
                true
            } else {
                false
            }
        }
    }

    pub static mut SIGINT_HOOK: Option<fn()> = None;

    pub fn set_sigint_hook(hook: fn()) {
        unsafe {
            SIGINT_HOOK = Some(hook);
        }
    }

    pub fn get_termios() -> Termios {
        unsafe { ACTIVE_TERMIOS }
    }

    pub fn set_termios(t: &Termios) {
        unsafe {
            ACTIVE_TERMIOS = *t;
        }
    }

    pub fn push_char(c: u8) -> bool {
        unsafe {
            let lflag = ACTIVE_TERMIOS.c_lflag;

            if (lflag & ISIG) != 0 && c == ACTIVE_TERMIOS.c_cc[VINTR] {
                LINE_LEN = 0;
                LAST_BACKSPACE_POPPED = false;
                if let Some(hook) = SIGINT_HOOK {
                    hook();
                }
                return false;
            }

            if (lflag & ICANON) != 0 {
                if c == ACTIVE_TERMIOS.c_cc[VEOF] {
                    LAST_BACKSPACE_POPPED = false;
                    if LINE_LEN == 0 {
                        EOF_PENDING = true;
                    } else {
                        #[allow(clippy::needless_range_loop)]
                        for i in 0..LINE_LEN {
                            let b = LINE_BUF[i];
                            let next_tail = (COOKED_TAIL + 1) % COOKED_BUF_SIZE;
                            if next_tail != COOKED_HEAD {
                                COOKED_QUEUE[COOKED_TAIL] = b;
                                COOKED_TAIL = next_tail;
                            }
                        }
                        LINE_LEN = 0;
                    }
                    return true;
                }

                if c == b'\n' && LAST_CHAR_WAS_CR {
                    LAST_CHAR_WAS_CR = false;
                    LAST_BACKSPACE_POPPED = false;
                    return false;
                }
                LAST_CHAR_WAS_CR = c == b'\r';

                if c == 0x08 || c == 0x7F || c == ACTIVE_TERMIOS.c_cc[VERASE] {
                    if LINE_LEN > 0 {
                        LINE_LEN -= 1;
                        LAST_BACKSPACE_POPPED = true;
                        true
                    } else {
                        LAST_BACKSPACE_POPPED = false;
                        false
                    }
                } else if c == b'\r' || c == b'\n' {
                    LAST_BACKSPACE_POPPED = false;
                    if LINE_LEN < LINE_BUF_SIZE {
                        LINE_BUF[LINE_LEN] = b'\n';
                        LINE_LEN += 1;
                    }
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..LINE_LEN {
                        let b = LINE_BUF[i];
                        let next_tail = (COOKED_TAIL + 1) % COOKED_BUF_SIZE;
                        if next_tail != COOKED_HEAD {
                            COOKED_QUEUE[COOKED_TAIL] = b;
                            COOKED_TAIL = next_tail;
                        }
                    }
                    LINE_LEN = 0;
                    true
                } else if LINE_LEN < LINE_BUF_SIZE - 1 {
                    LAST_BACKSPACE_POPPED = false;
                    LINE_BUF[LINE_LEN] = c;
                    LINE_LEN += 1;
                    true
                } else {
                    LAST_BACKSPACE_POPPED = false;
                    false
                }
            } else {
                LAST_BACKSPACE_POPPED = false;
                let next_tail = (COOKED_TAIL + 1) % COOKED_BUF_SIZE;
                if next_tail != COOKED_HEAD {
                    COOKED_QUEUE[COOKED_TAIL] = c;
                    COOKED_TAIL = next_tail;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn last_erase_succeeded() -> bool {
        unsafe { LAST_BACKSPACE_POPPED }
    }

    pub fn line_len() -> usize {
        unsafe { LINE_LEN }
    }

    pub fn read_tty(buf: &mut [u8]) -> usize {
        unsafe {
            let mut count = 0;
            while count < buf.len() && COOKED_HEAD != COOKED_TAIL {
                buf[count] = COOKED_QUEUE[COOKED_HEAD];
                COOKED_HEAD = (COOKED_HEAD + 1) % COOKED_BUF_SIZE;
                count += 1;
            }
            count
        }
    }

    pub fn flush_tty() {
        unsafe {
            COOKED_HEAD = 0;
            COOKED_TAIL = 0;
            LINE_LEN = 0;
            LAST_CHAR_WAS_CR = false;
            LAST_BACKSPACE_POPPED = false;
        }
    }
}

pub mod termios {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Termios {
        /// Input mode flags.
        pub c_iflag: u32,
        /// Output mode flags.
        pub c_oflag: u32,
        /// Control mode flags.
        pub c_cflag: u32,
        /// Local mode flags.
        pub c_lflag: u32,
        /// Line discipline.
        pub c_line: u8,
        /// Special control characters array.
        pub c_cc: [u8; 32],
        /// Input baud rate.
        pub c_ispeed: u32,
        /// Output baud rate.
        pub c_ospeed: u32,
    }

    impl Default for Termios {
        fn default() -> Self {
            Self {
                c_iflag: 0x0500,
                c_oflag: 0x0005,
                c_cflag: 0x00BF,
                c_lflag: 0x8A3B,
                c_line: 0,
                c_cc: [
                    3, 28, 127, 21, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0,
                ],
                c_ispeed: 38400,
                c_ospeed: 38400,
            }
        }
    }

    pub const ISIG: u32 = 0o0000001;

    pub const ICANON: u32 = 0o0000002;

    pub const ECHO: u32 = 0o0000010;

    pub const ECHOE: u32 = 0o0000020;

    pub const ECHOK: u32 = 0o0000040;

    pub const ECHONL: u32 = 0o0000100;

    pub const VINTR: usize = 0;

    pub const VQUIT: usize = 1;

    pub const VERASE: usize = 2;

    pub const VKILL: usize = 3;

    pub const VEOF: usize = 4;
}

pub use self::discipline::*;
pub use self::termios::*;

#[cfg(test)]
mod tests {
    use super::discipline::{get_termios, push_char, read_tty, set_sigint_hook, set_termios};
    use super::termios::{ICANON, ISIG};
    use core::sync::atomic::{AtomicBool, Ordering};

    static SIGINT_TRIGGERED: AtomicBool = AtomicBool::new(false);

    fn dummy_sigint() {
        SIGINT_TRIGGERED.store(true, Ordering::SeqCst);
    }

    #[test]
    fn test_canonical_line_editing() {
        let mut t = get_termios();
        t.c_lflag |= ICANON;
        set_termios(&t);

        // Drain buffer
        let mut sink = [0u8; 64];
        while read_tty(&mut sink) > 0 {}

        // Type "helxo", backspace, 'l', 'o', enter
        push_char(b'h');
        push_char(b'e');
        push_char(b'l');
        push_char(b'x');
        push_char(0x08); // Backspace
        push_char(b'l');
        push_char(b'o');

        // Not yet committed (no newline)
        let n = read_tty(&mut sink);
        assert_eq!(n, 0);

        // Commit newline
        push_char(b'\n');

        let n = read_tty(&mut sink);
        assert_eq!(n, 6);
        assert_eq!(&sink[..6], b"hello\n");
    }

    #[test]
    fn test_sigint_interception() {
        let mut t = get_termios();
        t.c_lflag |= ISIG;
        set_termios(&t);
        set_sigint_hook(dummy_sigint);

        SIGINT_TRIGGERED.store(false, Ordering::SeqCst);
        push_char(3); // Ctrl+C (VINTR = 3)

        assert!(SIGINT_TRIGGERED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_crlf_sequence_deduplication() {
        let mut t = get_termios();
        t.c_lflag |= ICANON;
        set_termios(&t);

        let mut sink = [0u8; 64];
        while read_tty(&mut sink) > 0 {}

        // Push "ok\r\n" - should yield exactly one committed line "ok\n"
        push_char(b'o');
        push_char(b'k');
        push_char(b'\r');
        push_char(b'\n');

        let n = read_tty(&mut sink);
        assert_eq!(n, 3);
        assert_eq!(&sink[..3], b"ok\n");

        // Buffer must be empty (no trailing phantom empty newline from \n)
        let n2 = read_tty(&mut sink);
        assert_eq!(n2, 0);
    }

    #[test]
    fn test_backspace_on_empty_line_preserves_prompt_boundary() {
        let mut t = get_termios();
        t.c_lflag |= ICANON;
        set_termios(&t);

        super::discipline::flush_tty();

        // Line is initially empty: backspace must return false and record no erase
        assert!(!push_char(0x08));
        assert!(!super::discipline::last_erase_succeeded());
        assert_eq!(super::discipline::line_len(), 0);

        // Also test 0x7F (DEL) on empty line
        assert!(!push_char(0x7F));
        assert!(!super::discipline::last_erase_succeeded());
        assert_eq!(super::discipline::line_len(), 0);

        // Push a character: line_len becomes 1
        assert!(push_char(b'a'));
        assert_eq!(super::discipline::line_len(), 1);

        // Backspace: pops 'a', line_len becomes 0, erase succeeds
        assert!(push_char(0x08));
        assert!(super::discipline::last_erase_succeeded());
        assert_eq!(super::discipline::line_len(), 0);

        // Backspace again on now empty line: must fail and prevent prompt deletion
        assert!(!push_char(0x08));
        assert!(!super::discipline::last_erase_succeeded());
        assert_eq!(super::discipline::line_len(), 0);
    }
}
