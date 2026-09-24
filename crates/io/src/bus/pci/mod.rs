// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Legacy PCI configuration space reader and device bus enumerator.

mod config;
mod device;
mod scanner;
#[cfg(test)]
mod tests;

pub use self::config::*;
pub use self::device::*;
pub use self::scanner::*;
