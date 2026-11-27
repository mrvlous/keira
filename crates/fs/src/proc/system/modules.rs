// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Dynamic `/proc/modules` node generating active kernel module status.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads active kernel modules table in Linux-compatible format into destination buffer.
pub fn read_modules(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut w = BufWriter::new(buf);
    let (mods, _) = keira_core::module::get_modules_snapshot();

    for m_opt in mods.iter().flatten() {
        let _ = writeln!(
            w,
            "{} {} {} - {} 0x{:016x}",
            m_opt.name_str(),
            m_opt.size,
            m_opt.ref_count,
            m_opt.state.as_str(),
            m_opt.load_address
        );
    }

    Ok(w.len())
}
