// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for command line history ring buffer.

#[cfg(test)]
pub mod test {
    use crate::history::*;
    use crate::state::session::*;

    #[test]
    fn test_history_push_and_count() {
        unsafe {
            BUFFER_LEN = 4;
            INPUT_BUFFER[0..4].copy_from_slice(b"help");
            let initial_count = HISTORY_COUNT;
            history_push();
            assert_eq!(HISTORY_COUNT, initial_count + 1);
            BUFFER_LEN = 0;
        }
    }
}
