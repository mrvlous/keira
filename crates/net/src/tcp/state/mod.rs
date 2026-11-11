// SPDX-License-Identifier: GPL-2.0-only

//! TCP connection states and port management.

pub mod machine;

pub use machine::{get_next_src_port, TcpState};
