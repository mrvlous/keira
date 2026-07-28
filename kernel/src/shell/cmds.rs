// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Commands Module Root
//!
//! Exposes all modular command handlers implemented in the `cmds/` directory.

pub mod copy;
pub mod cpu;
pub mod create;
pub mod delete;
pub mod devices;

pub mod disk;
pub mod drives;
pub mod edit;
pub mod folder;
pub mod go;
pub mod guide;
pub mod search;

pub mod hda;
pub mod help;
pub mod history;
pub mod initrd;
pub mod list;
pub mod login;
pub mod memory;
pub mod r#move;
pub mod play;
pub mod ramdisk;
pub mod reset;
pub mod run;
pub mod runtime;
pub mod say;
pub mod script;
pub mod system;
pub mod tasks;
pub mod theme;
pub mod time;
pub mod r#use;
pub mod view;
pub mod wait;
pub mod wipe;
pub mod write;
