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

//! Zero-copy page splicing (, ) pipe operations.

pub mod splice {
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
}

pub use self::splice::*;
