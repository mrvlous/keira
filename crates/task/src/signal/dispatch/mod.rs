// SPDX-License-Identifier: GPL-2.0-only

//! Signal routing and process delivery dispatch.

pub mod kill;

pub use kill::sys_kill;
