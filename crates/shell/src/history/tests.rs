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
