// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for POSIX priority message queues.

#[cfg(test)]
mod test {
    use crate::mqueue::*;

    #[test]
    fn test_mqueue_priority_ordering_and_unlink() {
        unsafe {
            let qname = "/test_queue_prio";
            let mqid = mq_open(qname, 0, 8, 128).expect("Failed to open mqueue");
            assert!(mqid < MAX_MQUEUES as u32);

            mq_send(qname, b"normal message", 5).expect("Send failed");
            mq_send(qname, b"urgent message", 20).expect("Send failed");

            let mut buf = [0u8; 128];
            let (len, prio) = mq_receive(qname, &mut buf).expect("Recv failed");
            assert_eq!(prio, 20);
            assert_eq!(&buf[..len], b"urgent message");

            let (len2, prio2) = mq_receive(qname, &mut buf).expect("Recv failed");
            assert_eq!(prio2, 5);
            assert_eq!(&buf[..len2], b"normal message");

            assert!(mq_receive(qname, &mut buf).is_err());
            mq_unlink(qname).expect("Unlink failed");
        }
    }
}
