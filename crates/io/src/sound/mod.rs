// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Audio hardware drivers (PC Speaker tone synthesizer and Intel High Definition Audio HDA).

pub mod hda;
pub mod speaker;

pub use hda as intel_hda;
pub use speaker::*;
