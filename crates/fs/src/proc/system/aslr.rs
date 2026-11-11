// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/sys/kernel/randomize_va_space` virtual telemetry provider.

use super::hooks::ASLR_POLICY_HOOK;

/// Synthesizes the Linux-compatible `/proc/sys/kernel/randomize_va_space` content.
pub fn read_randomize_va_space(buf: &mut [u8]) -> Result<usize, &'static str> {
    let policy = unsafe { ASLR_POLICY_HOOK.map(|f| f()).unwrap_or(2) };
    let content = match policy {
        0 => "0\n",
        1 => "1\n",
        _ => "2\n",
    };
    let bytes = content.as_bytes();
    if buf.len() < bytes.len() {
        return Err("Destination buffer too small for randomize_va_space");
    }
    buf[..bytes.len()].copy_from_slice(bytes);
    Ok(bytes.len())
}
