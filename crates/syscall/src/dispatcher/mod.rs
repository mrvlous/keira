// SPDX-License-Identifier: GPL-2.0-only

//! 64-bit fast syscall dispatcher, pointer validation and hardened userland system call router.

pub mod cleanup;
pub mod handlers;
pub mod router;
pub mod validate;

#[cfg(test)]
mod tests;

pub use cleanup::syscall_task_cleanup_hook;
pub use handlers::poll::{
    get_poll_telemetry, poll_telemetry_reset, PollFd, PollTelemetry, POLLERR, POLLHUP, POLLIN,
    POLLNVAL, POLLOUT, POLLPRI,
};
pub use handlers::select::{
    get_select_telemetry, handle_select, select_telemetry_reset, FdSet, SelectTelemetry, TimeVal,
    FD_SETSIZE,
};
pub use router::{sanitize_ret, syscall_dispatcher};
pub use validate::{validate_fd, HEAP_MAX_VADDR};
