// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for ICMP echo module.

use super::*;

#[test]
fn test_icmp_offline_ping() {
    unsafe {
        crate::driver::e1000::E1000_FOUND = false;
        assert!(send_ping("127.0.0.1").is_ok());
        let res = send_ping("10.0.2.2");
        assert!(res.is_err());
    }
}
