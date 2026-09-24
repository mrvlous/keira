// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Top-level integration and lifecycle tests for keira-ipc.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_shm_lifecycle_and_removal() {
        unsafe {
            let shmid = create_shm(4096).expect("Failed to create SHM");
            assert!(shmid < 4);

            let frame = get_shm_frame(shmid).expect("Failed to get SHM frame");
            assert_eq!(frame, 0x70000000 + (shmid as u64 * 0x1000));

            let table = get_shm_table();
            assert!(table[shmid].in_use);
            assert_eq!(table[shmid].size_bytes, 4096);

            remove_shm(shmid as u32).expect("Failed to remove SHM");
            let table = get_shm_table();
            assert!(!table[shmid].in_use);
        }
    }

    #[test]
    fn test_sem_lifecycle_and_removal() {
        unsafe {
            let semid = create_sem(0x99998888, 7).expect("Failed to create semaphore");
            assert!(semid < 4);

            let table = get_sem_table();
            assert!(table[semid as usize].in_use);
            assert_eq!(table[semid as usize].value, 7);

            remove_sem(semid).expect("Failed to remove semaphore");
            let table = get_sem_table();
            assert!(!table[semid as usize].in_use);
        }
    }

    #[test]
    fn test_mqueue_send_receive_unlink() {
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

    #[test]
    fn test_futex_wait_wake_requeue() {
        unsafe {
            futex_reset();
            let (_waits0, _wakes0, _requeues0, active0) = get_futex_stats();
            assert_eq!(active0, 0);

            assert_eq!(futex_wait(0x500000, 10, 2, 0xFFFFFFFF), Ok(0));
            assert_eq!(futex_wait(0x500000, 10, 3, 0xFFFFFFFF), Ok(0));
            assert_eq!(futex_wait(0x600000, 20, 4, 0xFFFFFFFF), Ok(0));

            let (_, _, _, active1) = get_futex_stats();
            assert_eq!(active1, 3);

            let req = futex_requeue(0x500000, 0x700000, 1).expect("Requeue failed");
            assert_eq!(req, 1);

            let woken = futex_wake(0x500000, 1, 0xFFFFFFFF).expect("Wake failed");
            assert_eq!(woken, 1);

            let woken2 = futex_wake(0x700000, 1, 0xFFFFFFFF).expect("Wake requeued failed");
            assert_eq!(woken2, 1);

            let woken3 = futex_wake(0x600000, 1, 0xFFFFFFFF).expect("Wake failed");
            assert_eq!(woken3, 1);

            let (_, _, _, active2) = get_futex_stats();
            assert_eq!(active2, 0);

            futex_reset();
        }
    }

    #[test]
    fn test_eventfd_create_write_read_close() {
        unsafe {
            let id = create_eventfd(10, 0).expect("Create EventFD failed");
            let table = get_eventfd_table();
            assert!(table
                .iter()
                .any(|e| e.in_use && e.id == id && e.count == 10));

            write_eventfd(id, 5).expect("Write failed");

            let val = read_eventfd(id).expect("Read failed");
            assert_eq!(val, 15);
            assert!(read_eventfd(id).is_err());

            let sem_id = create_eventfd(3, EFD_SEMAPHORE).expect("Create sem eventfd failed");
            assert_eq!(read_eventfd(sem_id), Ok(1));
            assert_eq!(read_eventfd(sem_id), Ok(1));
            assert_eq!(read_eventfd(sem_id), Ok(1));
            assert!(read_eventfd(sem_id).is_err());

            close_eventfd(id).expect("Close failed");
            close_eventfd(sem_id).expect("Close failed");
        }
    }
}
