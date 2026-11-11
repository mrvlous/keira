// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for signal definitions and job control table.

use super::*;

#[test]
fn test_posix_signal_constants() {
    assert_eq!(SIGHUP, 1);
    assert_eq!(SIGINT, 2);
    assert_eq!(SIGKILL, 9);
    assert_eq!(SIGSEGV, 11);
    assert_eq!(SIGTERM, 15);
    assert_eq!(SIGSTOP, 19);

    assert_eq!(SIG_BLOCK, 0);
    assert_eq!(SIG_UNBLOCK, 1);
    assert_eq!(SIG_SETMASK, 2);
}

#[test]
fn test_job_table_lifecycle() {
    unsafe {
        let jid = add_job(10, "test_job", true);
        assert!(jid > 0);

        let fg = get_foreground_job_pid();
        assert_eq!(fg, Some(10));

        remove_job_by_pid(10);
        let fg2 = get_foreground_job_pid();
        assert_ne!(fg2, Some(10));
    }
}
