// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Seccomp mode and state descriptors.

pub mod modes;

pub use modes::{SeccompMode, SeccompState, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT};
