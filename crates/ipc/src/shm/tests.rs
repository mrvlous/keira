// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for POSIX shared memory segments and semaphores.

#[cfg(test)]
mod test {
    use crate::shm::*;

    #[test]
    fn test_shm_allocation_and_frames() {
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
    fn test_semaphore_lifecycle() {
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
}
