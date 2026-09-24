// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Visual layout metrics, syntax detection, and full-screen renderer.

pub mod metrics;
pub mod syntax;
pub mod view;

pub use metrics::*;
pub use syntax::*;
pub use view::*;
