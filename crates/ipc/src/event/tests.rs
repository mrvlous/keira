// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for eventfd and epoll subsystems.

#[cfg(test)]
mod test {
    use crate::event::*;

    #[test]
    fn test_eventfd_operations() {
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

    #[test]
    fn test_epoll_lifecycle() {
        epoll_reset();
        let epfd = sys_epoll_create(10).expect("sys_epoll_create failed") as i32;
        assert_eq!(epfd, 100);

        let ev1 = EpollEvent {
            events: EPOLLIN,
            data: 42,
        };
        assert!(epoll_ctl_internal(epfd, EPOLL_CTL_ADD, 3, ev1).is_ok());

        let ev2 = EpollEvent {
            events: EPOLLIN | EPOLLOUT,
            data: 43,
        };
        assert!(epoll_ctl_internal(epfd, EPOLL_CTL_MOD, 3, ev2).is_ok());

        let (instances, items) = get_epoll_stats();
        assert_eq!(instances, 1);
        assert_eq!(items, 1);

        let mut events_buf = [EpollEvent::default(); 4];
        let ready =
            sys_epoll_wait(epfd, events_buf.as_mut_ptr() as u64, 4, 0).expect("wait failed");
        assert_eq!(ready, 1);
        assert_eq!(events_buf[0].data, 43);

        assert!(epoll_ctl_internal(epfd, EPOLL_CTL_DEL, 3, ev2).is_ok());
        let (_, items_after) = get_epoll_stats();
        assert_eq!(items_after, 0);
    }
}
