// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Log level severity and Linux syslog priority subsystem.

pub mod severity;

pub use severity::{
    LogLevel, KERN_ALERT, KERN_CRIT, KERN_DEBUG, KERN_EMERG, KERN_ERR, KERN_INFO, KERN_NOTICE,
    KERN_WARNING,
};
