// SPDX-License-Identifier: GPL-2.0-only

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
