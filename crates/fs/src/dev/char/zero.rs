// SPDX-License-Identifier: GPL-2.0-only

//! Pseudo character device `/dev/zero` providing infinite null bytes.

/// Reads from `/dev/zero`, filling the entire buffer with zeros.
pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    buf.fill(0);
    Ok(buf.len())
}

/// Writes to `/dev/zero`, discarding all data and returning bytes written.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
