// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Early boot orchestration, Multiboot2 parsing and architecture verification.

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
