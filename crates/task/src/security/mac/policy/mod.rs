// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control policy models, domains, and rule descriptors.

pub mod audit;
pub mod domain;
pub mod rule;

pub use audit::{MacAuditEvent, MacMode, MAC_AUDIT_LOG_CAPACITY};
pub use domain::MacDomain;
pub use rule::{MacRule, MAC_APPEND, MAC_EXEC, MAC_READ, MAC_WRITE, MAX_MAC_RULES};
