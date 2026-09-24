// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global kernel module static table storage pre-seeded with baseline drivers.

use super::super::core::descriptor::{KernelModule, MAX_MODULE_NAME};
use super::super::core::state::ModuleState;
use crate::sync::mutex::SpinMutex;

/// Maximum number of concurrently loaded kernel modules.
pub const MAX_MODULES: usize = 16;

/// Global kernel module table pre-seeded with baseline driver descriptors.
pub static MODULE_TABLE: SpinMutex<[Option<KernelModule>; MAX_MODULES]> = {
    const EMPTY: Option<KernelModule> = None;
    SpinMutex::new([
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'e';
                b[1] = b'x';
                b[2] = b't';
                b[3] = b'4';
                b[4] = b'_';
                b[5] = b'f';
                b[6] = b's';
                b
            },
            name_len: 7,
            size: 65536,
            state: ModuleState::Live,
            ref_count: 1,
            load_address: 0xFFFF_8000_0040_0000,
            description: "Native Linux EXT4 Filesystem Driver",
        }),
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'e';
                b[1] = b'1';
                b[2] = b'0';
                b[3] = b'0';
                b[4] = b'0';
                b[5] = b'_';
                b[6] = b'n';
                b[7] = b'i';
                b[8] = b'c';
                b
            },
            name_len: 9,
            size: 32768,
            state: ModuleState::Live,
            ref_count: 0,
            load_address: 0xFFFF_8000_0041_0000,
            description: "Intel 82540EM Gigabit Ethernet NIC Driver",
        }),
        Some(KernelModule {
            name: {
                let mut b = [0u8; MAX_MODULE_NAME];
                b[0] = b'a';
                b[1] = b'h';
                b[2] = b'c';
                b[3] = b'i';
                b[4] = b'_';
                b[5] = b's';
                b[6] = b'a';
                b[7] = b't';
                b[8] = b'a';
                b
            },
            name_len: 9,
            size: 24576,
            state: ModuleState::Live,
            ref_count: 2,
            load_address: 0xFFFF_8000_0042_0000,
            description: "Advanced Host Controller Interface SATA Driver",
        }),
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
        EMPTY,
    ])
};
