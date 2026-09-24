// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control security domains.

/// Security domain classifications for subjects (processes).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MacDomain {
    Kernel = 0,
    System = 1,
    User = 2,
    Guest = 3,
    Network = 4,
}

impl MacDomain {
    /// Retrieve string representation of security domain.
    pub fn as_str(&self) -> &'static str {
        match self {
            MacDomain::Kernel => "kernel",
            MacDomain::System => "system",
            MacDomain::User => "user",
            MacDomain::Guest => "guest",
            MacDomain::Network => "network",
        }
    }
}
