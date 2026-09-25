// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Command dispatcher, pipeline execution, redirection parser, and built-in runner.

pub mod dispatch;
pub mod env;
pub mod pipeline;

#[cfg(test)]
mod tests;

pub use dispatch::{
    count_pci_devices, execute_command, execute_command_inner, print_2digit, vga_init, RtcTime,
};
pub use env::expand_env_vars;
pub use pipeline::{
    execute_pipeline, execute_with_redirection, parse_input_redirection, parse_output_redirection,
};
