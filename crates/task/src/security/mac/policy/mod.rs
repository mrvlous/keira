// SPDX-License-Identifier: GPL-2.0-only

//! Mandatory Access Control policy models, domains and rule descriptors.

pub mod audit;
pub mod domain;
pub mod rule;

pub use audit::{MacAuditEvent, MacMode, MAC_AUDIT_LOG_CAPACITY};
pub use domain::MacDomain;
pub use rule::{MacRule, MAC_APPEND, MAC_EXEC, MAC_READ, MAC_WRITE, MAX_MAC_RULES};
