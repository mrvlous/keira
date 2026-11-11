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

//! Scalable I/O event notification multiplexer (`epoll`).

pub mod engine;
pub mod types;

pub use engine::{
    check_fd_readiness, epoll_ctl_internal, epoll_reset, get_epoll_instances, get_epoll_stats,
    sys_epoll_create, sys_epoll_ctl, sys_epoll_wait,
};
pub use types::{
    EpollEvent, EpollInstance, EpollItem, EPOLLERR, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLLPRI,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, MAX_EPOLL_INSTANCES, MAX_EPOLL_ITEMS,
};
