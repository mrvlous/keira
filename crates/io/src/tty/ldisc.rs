// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! N_TTY line discipline, canonical input editing and termios flags.

pub mod discipline {
    use core::sync::atomic::{AtomicBool, Ordering};
    use keira_core::sync::IrqMutex;

    use super::termios::{Termios, ICANON, ISIG, VEOF, VERASE, VINTR};

    pub const COOKED_BUF_SIZE: usize = 512;
    pub const LINE_BUF_SIZE: usize = 256;

    struct LdiscState {
        termios: Termios,
        cooked_queue: [u8; COOKED_BUF_SIZE],
        cooked_head: usize,
        cooked_tail: usize,
        line_buf: [u8; LINE_BUF_SIZE],
        line_len: usize,
        last_char_was_cr: bool,
        last_backspace_popped: bool,
        sigint_hook: Option<fn()>,
    }

    impl LdiscState {
        const fn new() -> Self {
            Self {
                termios: Termios {
                    c_iflag: 0x0500,
                    c_oflag: 0x0005,
                    c_cflag: 0x00BF,
                    c_lflag: 0x8A3B,
                    c_line: 0,
                    c_cc: [
                        3, 28, 127, 21, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0,
                    ],
                    c_ispeed: 38400,
                    c_ospeed: 38400,
                },
                cooked_queue: [0; COOKED_BUF_SIZE],
                cooked_head: 0,
                cooked_tail: 0,
                line_buf: [0; LINE_BUF_SIZE],
                line_len: 0,
                last_char_was_cr: false,
                last_backspace_popped: false,
                sigint_hook: None,
            }
        }

        fn push_cooked_byte(&mut self, b: u8) -> bool {
            let next_tail = (self.cooked_tail + 1) % COOKED_BUF_SIZE;
            if next_tail != self.cooked_head {
                self.cooked_queue[self.cooked_tail] = b;
                self.cooked_tail = next_tail;
                true
            } else {
                false
            }
        }

        fn flush_line_buf(&mut self) {
            for i in 0..self.line_len {
                let b = self.line_buf[i];
                let _ = self.push_cooked_byte(b);
            }
            self.line_len = 0;
        }
    }

    static LDISC: IrqMutex<LdiscState> = IrqMutex::new(LdiscState::new());
    static EOF_PENDING: AtomicBool = AtomicBool::new(false);

    pub fn take_eof_pending() -> bool {
        EOF_PENDING.swap(false, Ordering::SeqCst)
    }

    pub fn set_sigint_hook(hook: fn()) {
        LDISC.lock().sigint_hook = Some(hook);
    }

    pub fn get_termios() -> Termios {
        LDISC.lock().termios
    }

    pub fn set_termios(t: &Termios) {
        let mut state = LDISC.lock();
        let was_canon = (state.termios.c_lflag & ICANON) != 0;
        let now_canon = (t.c_lflag & ICANON) != 0;
        // Flush any typed-ahead canonical bytes before switching to raw mode so
        // earlier characters cannot stay stranded and reorder behind later reads.
        if was_canon && !now_canon && state.line_len > 0 {
            state.flush_line_buf();
            state.last_char_was_cr = false;
        }
        state.termios = *t;
    }

    pub fn push_char(c: u8) -> bool {
        let hook_to_run: Option<fn()>;
        let accepted = {
            let mut state = LDISC.lock();
            let lflag = state.termios.c_lflag;

            if (lflag & ISIG) != 0 && c == state.termios.c_cc[VINTR] {
                state.line_len = 0;
                state.last_backspace_popped = false;
                hook_to_run = state.sigint_hook;
                false
            } else {
                hook_to_run = None;
                if (lflag & ICANON) != 0 {
                    if c == state.termios.c_cc[VEOF] {
                        state.last_backspace_popped = false;
                        if state.line_len == 0 {
                            EOF_PENDING.store(true, Ordering::SeqCst);
                        } else {
                            state.flush_line_buf();
                        }
                        true
                    } else if c == b'\n' && state.last_char_was_cr {
                        state.last_char_was_cr = false;
                        state.last_backspace_popped = false;
                        false
                    } else {
                        state.last_char_was_cr = c == b'\r';

                        if c == 0x08 || c == 0x7F || c == state.termios.c_cc[VERASE] {
                            if state.line_len > 0 {
                                state.line_len -= 1;
                                state.last_backspace_popped = true;
                                true
                            } else {
                                state.last_backspace_popped = false;
                                false
                            }
                        } else if c == b'\r' || c == b'\n' {
                            state.last_backspace_popped = false;
                            if state.line_len < LINE_BUF_SIZE {
                                let idx = state.line_len;
                                state.line_buf[idx] = b'\n';
                                state.line_len = idx + 1;
                            }
                            state.flush_line_buf();
                            true
                        } else if state.line_len < LINE_BUF_SIZE - 1 {
                            state.last_backspace_popped = false;
                            let idx = state.line_len;
                            state.line_buf[idx] = c;
                            state.line_len = idx + 1;
                            true
                        } else {
                            state.last_backspace_popped = false;
                            false
                        }
                    }
                } else {
                    state.last_backspace_popped = false;
                    state.push_cooked_byte(c)
                }
            }
        };

        if let Some(hook) = hook_to_run {
            hook();
        }
        accepted
    }

    pub fn last_erase_succeeded() -> bool {
        LDISC.lock().last_backspace_popped
    }

    pub fn line_len() -> usize {
        LDISC.lock().line_len
    }

    pub fn read_tty(buf: &mut [u8]) -> usize {
        let mut state = LDISC.lock();
        let mut count = 0;
        while count < buf.len() && state.cooked_head != state.cooked_tail {
            buf[count] = state.cooked_queue[state.cooked_head];
            state.cooked_head = (state.cooked_head + 1) % COOKED_BUF_SIZE;
            count += 1;
        }
        count
    }

    pub fn flush_tty() {
        let mut state = LDISC.lock();
        state.cooked_head = 0;
        state.cooked_tail = 0;
        state.line_len = 0;
        state.last_char_was_cr = false;
        state.last_backspace_popped = false;
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

    #[test]
    fn test_canonical_to_raw_flushes_pending_input() {
        let mut canon = get_termios();
        canon.c_lflag |= ICANON;
        set_termios(&canon);
        super::discipline::flush_tty();

        // Simulate type-ahead before interactive readline turns ICANON off.
        assert!(push_char(b's'));
        assert_eq!(super::discipline::line_len(), 1);

        let mut raw = canon;
        raw.c_lflag &= !ICANON;
        set_termios(&raw);
        assert_eq!(super::discipline::line_len(), 0);

        for &b in b"u 1000\n" {
            assert!(push_char(b));
        }

        let mut out = [0u8; 16];
        let n = read_tty(&mut out);
        assert_eq!(n, 8);
        assert_eq!(&out[..n], b"su 1000\n");

        let mut extra = [0u8; 4];
        let n2 = read_tty(&mut extra);
        assert_eq!(n2, 0);
    }
}
