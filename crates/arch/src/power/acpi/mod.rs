// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Advanced Configuration and Power Interface (ACPI) subsystem.

pub mod shutdown;
pub mod tables;

pub use shutdown::*;
pub use tables::*;

#[cfg(test)]
mod tests;
