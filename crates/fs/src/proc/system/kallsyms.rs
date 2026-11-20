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

//! Dynamic `/proc/kallsyms` node generating exported kernel symbols.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads exported kernel symbols table in Linux-compatible format into destination buffer.
pub fn read_kallsyms(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut w = BufWriter::new(buf);
    let symbols = &keira_core::module::symbol::table::BASE_KALLSYMS;

    for sym in symbols.iter() {
        let sym_type = if sym.is_gpl_only { "T" } else { "t" };
        let _ = writeln!(w, "{:016x} {} {}", sym.addr, sym_type, sym.name);
    }

    Ok(w.len())
}
