// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early boot orchestration, Multiboot2 parsing, and architecture verification.

pub mod cpu;
pub mod early;
pub mod multiboot;

#[cfg(test)]
mod tests;

pub use cpu::detect_cpu;
pub use early::early_bringup;
pub use multiboot::{
    parse_multiboot2, BootPayloadInfo, MULTIBOOT2_BOOTLOADER_MAGIC, TAG_TYPE_END,
    TAG_TYPE_FRAMEBUFFER, TAG_TYPE_MODULE,
};
