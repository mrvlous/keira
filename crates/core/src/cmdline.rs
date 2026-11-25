// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
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

//! Kernel boot command-line storage and argument parsing helpers.

use crate::sync::IrqMutex;

/// Maximum length of the stored Multiboot2 kernel command line string.
pub const MAX_CMDLINE_LEN: usize = 256;

const DEFAULT_CMDLINE: &[u8] = b"console=tty0 quiet";

#[derive(Clone, Copy)]
struct BootCmdlineState {
    buf: [u8; MAX_CMDLINE_LEN],
    len: usize,
    explicitly_set: bool,
}

impl BootCmdlineState {
    const fn new() -> Self {
        Self {
            buf: [0u8; MAX_CMDLINE_LEN],
            len: 0,
            explicitly_set: false,
        }
    }

    fn active_bytes(&self) -> &[u8] {
        if self.explicitly_set && self.len > 0 {
            &self.buf[..self.len]
        } else {
            DEFAULT_CMDLINE
        }
    }
}

static BOOT_CMDLINE: IrqMutex<BootCmdlineState> = IrqMutex::new(BootCmdlineState::new());

/// Stores the kernel boot command line provided by the bootloader.
pub fn set_boot_cmdline(raw: &str) {
    let trimmed = raw.trim();
    let bytes = trimmed.as_bytes();
    let copy_len = bytes.len().min(MAX_CMDLINE_LEN);

    let mut state = BOOT_CMDLINE.lock();
    state.buf = [0u8; MAX_CMDLINE_LEN];
    state.buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
    state.len = copy_len;
    state.explicitly_set = true;
}

/// Clears any stored boot command line back to the built-in default state.
pub fn reset_boot_cmdline() {
    let mut state = BOOT_CMDLINE.lock();
    *state = BootCmdlineState::new();
}

/// Returns whether a bootloader command line has been explicitly registered.
pub fn is_boot_cmdline_set() -> bool {
    let state = BOOT_CMDLINE.lock();
    state.explicitly_set && state.len > 0
}

/// Copies the active kernel command line into `out` and returns the byte count written.
pub fn copy_boot_cmdline(out: &mut [u8]) -> usize {
    let state = BOOT_CMDLINE.lock();
    let src = state.active_bytes();
    let to_copy = src.len().min(out.len());
    out[..to_copy].copy_from_slice(&src[..to_copy]);
    to_copy
}

/// Returns `true` if `flag` matches a whitespace-delimited token in the boot command line.
pub fn has_boot_cmdline_flag(flag: &str) -> bool {
    let mut local = [0u8; MAX_CMDLINE_LEN];
    let len = copy_boot_cmdline(&mut local);
    if let Ok(s) = core::str::from_utf8(&local[..len]) {
        return s.split_whitespace().any(|tok| tok == flag);
    }
    false
}

/// Looks up `key=<value>` on the boot command line, copying `<value>` into `out`.
pub fn get_boot_cmdline_value<'a>(key: &str, out: &'a mut [u8]) -> Option<&'a str> {
    let mut local = [0u8; MAX_CMDLINE_LEN];
    let len = copy_boot_cmdline(&mut local);
    let s = core::str::from_utf8(&local[..len]).ok()?;

    for tok in s.split_whitespace() {
        if let Some((k, v)) = tok.split_once('=') {
            if k == key && !v.is_empty() {
                let vb = v.as_bytes();
                let n = vb.len().min(out.len());
                out[..n].copy_from_slice(&vb[..n]);
                return core::str::from_utf8(&out[..n]).ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_cmdline_set_parse_and_reset() {
        reset_boot_cmdline();
        assert!(!is_boot_cmdline_set());

        let mut buf = [0u8; MAX_CMDLINE_LEN];
        let n = copy_boot_cmdline(&mut buf);
        assert_eq!(&buf[..n], b"console=tty0 quiet");

        set_boot_cmdline("  console=ring0 init=/bin/sh.elf quiet  ");
        assert!(is_boot_cmdline_set());
        assert!(has_boot_cmdline_flag("console=ring0"));
        assert!(has_boot_cmdline_flag("quiet"));
        assert!(!has_boot_cmdline_flag("nosmp"));

        let mut val_buf = [0u8; 64];
        assert_eq!(
            get_boot_cmdline_value("init", &mut val_buf),
            Some("/bin/sh.elf")
        );
        assert_eq!(get_boot_cmdline_value("root", &mut val_buf), None);

        reset_boot_cmdline();
    }
}
