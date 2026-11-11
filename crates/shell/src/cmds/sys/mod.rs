// SPDX-License-Identifier: GPL-2.0-only

//! System configuration, telemetry, services and power shell commands.

pub mod control;
pub mod daemon;
pub mod info;

#[cfg(test)]
mod tests;

pub use control::{power, reset, sync, wipe};
pub use daemon::{syslog, watchpoint};
pub use info::{
    aslrinfo, cpu, irq, memory, pipes, pollinfo, regs, rlimit, rnginfo, secinfo, selectinfo,
    slabinfo, smp, system, unixsock, unwind, vmainfo,
};
