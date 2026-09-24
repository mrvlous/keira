// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-specific initial user stack formatting.

pub mod amd64;
pub mod i386;

pub use amd64::setup_user_stack_64;
pub use i386::setup_user_stack_32;
