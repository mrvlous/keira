// SPDX-License-Identifier: GPL-2.0-only

//! Network device drivers and stack initialization stage.

pub mod driver;

pub use driver::init_network;
