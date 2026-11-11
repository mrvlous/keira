// SPDX-License-Identifier: GPL-2.0-only

//! Pseudo character devices `/dev/random` and `/dev/urandom` providing entropy.

/// Fills destination buffer with hardware-harvested silicon entropy.
pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut i = 0;
    while i < buf.len() {
        let entropy = keira_arch::cpu::get_entropy_u32();
        let bytes = entropy.to_ne_bytes();
        let chunk = (buf.len() - i).min(4);
        buf[i..i + chunk].copy_from_slice(&bytes[..chunk]);
        i += chunk;
    }
    Ok(buf.len())
}

/// Discards entropy seed writes to random/urandom.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
