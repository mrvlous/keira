// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Command dispatching, execution entry, and built-in router handlers.

pub mod entry;
pub mod hardware;
pub mod router;

pub use entry::*;
pub use hardware::*;
pub use router::*;
