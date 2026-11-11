// SPDX-License-Identifier: GPL-2.0-only

//! Runtime execution, userspace context switching and terminal loop.

pub mod main_loop;
pub mod userspace;

#[cfg(test)]
mod tests;

pub use main_loop::enter_main_loop;
pub use userspace::enter_userspace;
