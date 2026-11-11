// SPDX-License-Identifier: GPL-2.0-only

//! Internet Control Message Protocol (ICMP) network diagnostics.

pub mod echo;

#[cfg(test)]
mod tests;

pub use echo::send_ping;

pub mod ping {
    pub use super::echo::send_ping;
}
