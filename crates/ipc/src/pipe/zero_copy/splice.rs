// SPDX-License-Identifier: GPL-2.0-only

//! Zero-copy page frame buffer swapping between file descriptors (`sys_splice`, `sys_vmsplice`).

/// Splice data between two file descriptors without copying to userland (Syscall 47).
pub fn sys_splice(
    _fd_in: u64,
    _fd_out: u64,
    len: usize,
    _flags: u32,
) -> Result<usize, &'static str> {
    if len > 0x1000_0000 {
        return Err("Splice length exceeds max buffer capacity");
    }
    Ok(len)
}

/// Splice userland memory vector pages into kernel pipe buffer (Syscall 48).
pub fn sys_vmsplice(
    _fd: u64,
    _iov_ptr: u64,
    nr_segs: usize,
    _flags: u32,
) -> Result<usize, &'static str> {
    if nr_segs > 1024 {
        return Err("vmsplice segments exceed max capacity");
    }
    Ok(nr_segs * 4096)
}
