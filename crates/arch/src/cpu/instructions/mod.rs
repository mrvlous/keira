// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Low-level x86/x86_64 CPU assembly intrinsics subsystem.

pub mod intrinsics;

pub use intrinsics::{cli, hlt, invlpg, pause, rdtsc, sti};
