// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic resolution configuration and memory base setup for the linear framebuffer.

use super::state::{FB_ACTIVE, FB_ADDR, FB_HEIGHT, FB_PITCH, FB_WIDTH};

/// Configures dynamic linear framebuffer geometry and physical address.
///
/// # Safety
///
/// Modifies global framebuffer static variables. Caller must ensure `addr` points to valid memory.
pub unsafe fn set_resolution(width: u32, height: u32, pitch: u32, addr: u64) {
    FB_WIDTH = width;
    FB_HEIGHT = height;
    FB_PITCH = pitch;
    if addr != 0 {
        FB_ADDR = addr;
        FB_ACTIVE = true;
    }
}
