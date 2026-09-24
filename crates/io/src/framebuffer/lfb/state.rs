// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global state descriptors and resolution parameters for the 32-bpp linear framebuffer.

/// Physical or virtual base memory address of the linear framebuffer.
pub static mut FB_ADDR: u64 = 0xFD000000;

/// Horizontal resolution in pixels.
pub static mut FB_WIDTH: u32 = 1024;

/// Vertical resolution in pixels.
pub static mut FB_HEIGHT: u32 = 768;

/// Stride in bytes per scanline.
pub static mut FB_PITCH: u32 = 4096;

/// Color depth in bits per pixel.
pub static mut FB_BPP: u8 = 32;

/// Flag indicating whether the linear framebuffer graphics device is active.
pub static mut FB_ACTIVE: bool = false;
