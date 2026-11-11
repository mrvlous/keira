// SPDX-License-Identifier: GPL-2.0-only

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
