// SPDX-License-Identifier: GPL-2.0-only

//! Scalable event multiplexing (`epoll`, `eventfd`, `signalfd`).

pub mod epoll;
pub mod eventfd;

pub use epoll::*;
pub use eventfd::*;

#[cfg(test)]
mod tests;
