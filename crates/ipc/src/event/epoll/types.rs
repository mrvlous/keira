// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Event poll (`epoll`) types, event flags, and interest list descriptors.

pub const EPOLLIN: u32 = 0x0001;
pub const EPOLLPRI: u32 = 0x0002;
pub const EPOLLOUT: u32 = 0x0004;
pub const EPOLLERR: u32 = 0x0008;
pub const EPOLLHUP: u32 = 0x0010;
pub const EPOLLET: u32 = 1 << 31;

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

pub const MAX_EPOLL_INSTANCES: usize = 16;
pub const MAX_EPOLL_ITEMS: usize = 32;

/// Linux ABI-compatible epoll event notification payload.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

/// Registered file descriptor interest entry inside an epoll instance.
#[derive(Copy, Clone, Debug, Default)]
pub struct EpollItem {
    pub fd: i32,
    pub event: EpollEvent,
    pub ready_events: u32,
    pub in_use: bool,
}

/// Active epoll instance maintaining an interest list of monitored descriptors.
#[derive(Copy, Clone, Debug)]
pub struct EpollInstance {
    pub epfd: i32,
    pub items: [EpollItem; MAX_EPOLL_ITEMS],
    pub item_count: usize,
    pub active: bool,
    pub total_polls: u64,
}
