// SPDX-License-Identifier: GPL-2.0-only

//! Pseudo character device `/dev/null` discarding writes and returning EOF on reads.

/// Reads from `/dev/null`, instantly returning 0 bytes (EOF).
pub fn read(_buf: &mut [u8]) -> Result<usize, &'static str> {
    Ok(0)
}

/// Writes to `/dev/null`, discarding all data and returning bytes written.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
