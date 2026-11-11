// SPDX-License-Identifier: GPL-2.0-only

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
