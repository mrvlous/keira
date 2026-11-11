// SPDX-License-Identifier: GPL-2.0-only

//! Process identity, permissions and credential descriptors.

pub mod creds;

pub use creds::{
    get_current_egid, get_current_euid, get_current_gid, get_current_uid, set_current_gid,
    set_current_uid,
};
