// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/mounts` node reporting registered filesystem mounts.

/// Reads active filesystem mount points into destination buffer.
pub fn read_mounts(buf: &mut [u8]) -> Result<usize, &'static str> {
    let len = crate::vfs::mount::format_proc_mounts(buf);
    Ok(len)
}
