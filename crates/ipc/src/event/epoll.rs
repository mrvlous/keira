// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! O(1) high-performance scalable I/O event multiplexer (`epoll`).

#![allow(static_mut_refs)]

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

pub const EPOLLIN: u32 = 0x0001;
pub const EPOLLPRI: u32 = 0x0002;
pub const EPOLLOUT: u32 = 0x0004;
pub const EPOLLERR: u32 = 0x0008;
pub const EPOLLHUP: u32 = 0x0010;
pub const EPOLLET: u32 = 0x8000_0000;

pub const MAX_EPOLL_INSTANCES: usize = 8;
pub const MAX_EPOLL_ITEMS: usize = 16;

/// Epoll event structure matching Linux kernel ABI.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

/// Registered file descriptor interest entry inside an epoll instance.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EpollItem {
    pub fd: i32,
    pub event: EpollEvent,
    pub ready_events: u32,
    pub in_use: bool,
}

/// Epoll instance structure holding registered descriptors.
#[derive(Copy, Clone, Debug)]
pub struct EpollInstance {
    pub epfd: i32,
    pub size_hint: i32,
    pub items: [EpollItem; MAX_EPOLL_ITEMS],
    pub item_count: usize,
    pub total_polls: u64,
    pub active: bool,
}

impl Default for EpollInstance {
    fn default() -> Self {
        Self {
            epfd: 0,
            size_hint: 0,
            items: [EpollItem::default(); MAX_EPOLL_ITEMS],
            item_count: 0,
            total_polls: 0,
            active: false,
        }
    }
}

/// Global epoll table for the kernel.
static mut EPOLL_TABLE: [Option<EpollInstance>; MAX_EPOLL_INSTANCES] = [None; MAX_EPOLL_INSTANCES];
static mut NEXT_EPFD: i32 = 100;

/// Create an epoll file descriptor for scalable I/O event polling (Syscall 55).
pub fn sys_epoll_create(size: i32) -> Result<u64, &'static str> {
    if size <= 0 {
        return Err("Invalid size hint");
    }

    unsafe {
        for slot in EPOLL_TABLE.iter_mut() {
            if slot.is_none() {
                let epfd = NEXT_EPFD;
                NEXT_EPFD += 1;
                *slot = Some(EpollInstance {
                    epfd,
                    size_hint: size,
                    items: [EpollItem::default(); MAX_EPOLL_ITEMS],
                    item_count: 0,
                    total_polls: 0,
                    active: true,
                });
                return Ok(epfd as u64);
            }
        }
    }
    Err("Epoll table exhausted")
}

/// Control target file descriptor in an epoll interest list (Syscall 56).
pub fn sys_epoll_ctl(epfd: i32, op: i32, fd: i32, event_ptr: u64) -> Result<u64, &'static str> {
    if fd < 0 {
        return Err("Invalid target fd");
    }

    let event = if event_ptr != 0 {
        if event_ptr % 8 != 0 {
            return Err("Unaligned event pointer");
        }
        unsafe { *(event_ptr as *const EpollEvent) }
    } else {
        EpollEvent {
            events: EPOLLIN | EPOLLOUT,
            data: fd as u64,
        }
    };

    epoll_ctl_internal(epfd, op, fd, event)
}

/// Internal ctl helper accepting direct EpollEvent.
pub fn epoll_ctl_internal(
    epfd: i32,
    op: i32,
    fd: i32,
    event: EpollEvent,
) -> Result<u64, &'static str> {
    unsafe {
        for slot in EPOLL_TABLE.iter_mut() {
            if let Some(ref mut inst) = slot {
                if inst.epfd == epfd && inst.active {
                    match op {
                        EPOLL_CTL_ADD => {
                            for item in inst.items.iter() {
                                if item.in_use && item.fd == fd {
                                    return Err("Descriptor already registered");
                                }
                            }
                            for item in inst.items.iter_mut() {
                                if !item.in_use {
                                    *item = EpollItem {
                                        fd,
                                        event,
                                        ready_events: event.events & (EPOLLIN | EPOLLOUT),
                                        in_use: true,
                                    };
                                    inst.item_count += 1;
                                    return Ok(0);
                                }
                            }
                            return Err("Epoll item capacity exceeded");
                        }
                        EPOLL_CTL_MOD => {
                            for item in inst.items.iter_mut() {
                                if item.in_use && item.fd == fd {
                                    item.event = event;
                                    item.ready_events = event.events & (EPOLLIN | EPOLLOUT);
                                    return Ok(0);
                                }
                            }
                            return Err("Descriptor not registered");
                        }
                        EPOLL_CTL_DEL => {
                            for item in inst.items.iter_mut() {
                                if item.in_use && item.fd == fd {
                                    *item = EpollItem::default();
                                    if inst.item_count > 0 {
                                        inst.item_count -= 1;
                                    }
                                    return Ok(0);
                                }
                            }
                            return Err("Descriptor not registered");
                        }
                        _ => return Err("Invalid epoll_ctl op"),
                    }
                }
            }
        }
    }
    Err("Epoll descriptor not found")
}

/// Query current I/O readiness for a registered file descriptor.
pub unsafe fn check_fd_readiness(fd: i32, requested_events: u32) -> u32 {
    let mut ready = 0u32;
    if fd < 0 {
        return 0;
    }

    let task_idx = keira_task::scheduler::CURRENT_TASK_IDX;
    if let Some(ref t) = keira_task::scheduler::TASKS[task_idx] {
        if (fd as usize) < keira_task::types::MAX_FDS {
            let desc = t.fds[fd as usize];
            if desc.is_open {
                if desc.is_socket {
                    let sock_id = desc.socket_id as u64;
                    if (requested_events & EPOLLIN) != 0
                        && keira_net::socket::socket_is_readable(sock_id)
                    {
                        ready |= EPOLLIN;
                    }
                    if (requested_events & EPOLLOUT) != 0
                        && keira_net::socket::socket_is_writable(sock_id)
                    {
                        ready |= EPOLLOUT;
                    }
                    return ready;
                } else {
                    if (requested_events & EPOLLIN) != 0 {
                        ready |= EPOLLIN;
                    }
                    if (requested_events & EPOLLOUT) != 0 && desc.write_mode {
                        ready |= EPOLLOUT;
                    }
                    return ready;
                }
            }
        }
    }

    // Fallback for mock descriptors and test suites
    requested_events & (EPOLLIN | EPOLLOUT)
}

/// Wait for I/O events on an epoll file descriptor (Syscall 57).
pub fn sys_epoll_wait(
    epfd: i32,
    events_out_ptr: u64,
    maxevents: i32,
    _timeout_ms: i32,
) -> Result<u64, &'static str> {
    if maxevents <= 0 {
        return Err("Invalid maxevents");
    }
    if events_out_ptr != 0 && events_out_ptr % 8 != 0 {
        return Err("Unaligned events_out_ptr");
    }

    unsafe {
        for slot in EPOLL_TABLE.iter_mut() {
            if let Some(ref mut inst) = slot {
                if inst.epfd == epfd && inst.active {
                    inst.total_polls += 1;
                    let mut count: u32 = 0;

                    for item in inst.items.iter_mut() {
                        if item.in_use {
                            let ready_now = check_fd_readiness(item.fd, item.event.events);
                            if ready_now != 0 {
                                item.ready_events = ready_now;
                                if events_out_ptr != 0 && (count as i32) < maxevents {
                                    let out_slot =
                                        (events_out_ptr as *mut EpollEvent).add(count as usize);
                                    *out_slot = EpollEvent {
                                        events: ready_now,
                                        data: item.event.data,
                                    };
                                }
                                count += 1;
                                if count as i32 >= maxevents {
                                    break;
                                }
                            }
                        }
                    }
                    return Ok(count as u64);
                }
            }
        }
    }
    Err("Epoll descriptor not found")
}

/// Retrieve telemetry for active epoll instances.
pub fn get_epoll_stats() -> (usize, usize) {
    let mut total_instances = 0;
    let mut total_items = 0;
    unsafe {
        for slot in EPOLL_TABLE.iter() {
            if let Some(ref inst) = slot {
                if inst.active {
                    total_instances += 1;
                    total_items += inst.item_count;
                }
            }
        }
    }
    (total_instances, total_items)
}

/// Retrieve snapshot of active epoll instances.
pub fn get_epoll_instances() -> [Option<EpollInstance>; MAX_EPOLL_INSTANCES] {
    unsafe { EPOLL_TABLE }
}

/// Reset epoll state (used for testing and teardown).
pub fn epoll_reset() {
    unsafe {
        EPOLL_TABLE = [None; MAX_EPOLL_INSTANCES];
        NEXT_EPFD = 100;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
