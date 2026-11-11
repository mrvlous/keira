// SPDX-License-Identifier: GPL-2.0-only

//! Filesystem and storage mount shell commands.

pub mod mount;

#[cfg(test)]
mod tests;

pub use mount::dcache;
pub use mount::disk;
pub use mount::icache;
