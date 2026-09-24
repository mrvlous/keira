// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! TTY line discipline canonical input queue, line editing, and signal interception.

use super::termios::{Termios, ICANON, ISIG, VERASE, VINTR};

static mut ACTIVE_TERMIOS: Termios = Termios {
    c_iflag: 0x0500,
    c_oflag: 0x0005,
    c_cflag: 0x00BF,
    c_lflag: 0x8A3B,
    c_line: 0,
    c_cc: [
        3, 28, 127, 21, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0,
    ],
    c_ispeed: 38400,
    c_ospeed: 38400,
};

/// Circular buffer capacity for cooked (committed) terminal input characters.
pub const COOKED_BUF_SIZE: usize = 512;
static mut COOKED_QUEUE: [u8; COOKED_BUF_SIZE] = [0; COOKED_BUF_SIZE];
static mut COOKED_HEAD: usize = 0;
static mut COOKED_TAIL: usize = 0;

/// Buffer capacity for in-progress uncommitted line editing.
pub const LINE_BUF_SIZE: usize = 256;
static mut LINE_BUF: [u8; LINE_BUF_SIZE] = [0; LINE_BUF_SIZE];
static mut LINE_LEN: usize = 0;

/// Hook function invoked when a SIGINT character is intercepted in canonical mode.
pub static mut SIGINT_HOOK: Option<fn()> = None;

/// Registers an interrupt callback hook invoked upon receiving SIGINT (Ctrl+C).
pub fn set_sigint_hook(hook: fn()) {
    unsafe {
        SIGINT_HOOK = Some(hook);
    }
}

/// Retrieves a copy of the current active termios configuration.
pub fn get_termios() -> Termios {
    unsafe { ACTIVE_TERMIOS }
}

/// Applies a new termios configuration.
pub fn set_termios(t: &Termios) {
    unsafe {
        ACTIVE_TERMIOS = *t;
    }
}

/// Pushes a raw input character into the TTY line discipline state machine.
pub fn push_char(c: u8) {
    unsafe {
        let lflag = ACTIVE_TERMIOS.c_lflag;

        if (lflag & ISIG) != 0 && c == ACTIVE_TERMIOS.c_cc[VINTR] {
            if let Some(hook) = SIGINT_HOOK {
                hook();
            }
            return;
        }

        if (lflag & ICANON) != 0 {
            if c == 0x08 || c == 0x7F || c == ACTIVE_TERMIOS.c_cc[VERASE] {
                if LINE_LEN > 0 {
                    LINE_LEN -= 1;
                }
            } else if c == b'\r' || c == b'\n' {
                if LINE_LEN < LINE_BUF_SIZE {
                    LINE_BUF[LINE_LEN] = b'\n';
                    LINE_LEN += 1;
                }
                for i in 0..LINE_LEN {
                    let b = LINE_BUF[i];
                    let next_tail = (COOKED_TAIL + 1) % COOKED_BUF_SIZE;
                    if next_tail != COOKED_HEAD {
                        COOKED_QUEUE[COOKED_TAIL] = b;
                        COOKED_TAIL = next_tail;
                    }
                }
                LINE_LEN = 0;
            } else if LINE_LEN < LINE_BUF_SIZE - 1 {
                LINE_BUF[LINE_LEN] = c;
                LINE_LEN += 1;
            }
        } else {
            let next_tail = (COOKED_TAIL + 1) % COOKED_BUF_SIZE;
            if next_tail != COOKED_HEAD {
                COOKED_QUEUE[COOKED_TAIL] = c;
                COOKED_TAIL = next_tail;
            }
        }
    }
}

/// Reads cooked characters from the canonical line buffer into the target destination slice.
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
