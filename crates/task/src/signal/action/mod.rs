// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Signal action handler registration and process disposition tracking.

pub mod disposition;

pub use disposition::{
    get_signal_handler, reset_signal_handlers, sys_sigaction, MAX_SIGNAL_TASKS, SIGNAL_HANDLERS,
};
