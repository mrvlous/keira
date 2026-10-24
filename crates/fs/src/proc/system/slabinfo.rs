// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/slabinfo` node reporting kernel slab allocator statistics.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted slab allocator cache metrics into destination buffer.
pub fn read_slabinfo(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let _ = write!(
        writer,
        "slabinfo - version: 2.1\n\
         # name            <active_objs> <num_objs> <objsize> <objperslab> <pagesperslab> : tunables <limit> <batchcount> <sharedfactor> : slabdata <active_slabs> <num_slabs> <sharedavail>\n"
    );

    for cache in keira_mem::get_all_kmem_caches() {
        let active_objs = cache.allocated_count();
        let num_objs = cache.total_count();
        let obj_size = cache.obj_size();
        let objs_per_slab = cache.objs_per_slab();
        let pages_per_slab = cache.pages_per_slab();
        let limit = cache.limit_tunable();
        let batchcount = cache.batchcount_tunable();
        let sharedfactor = cache.sharedfactor_tunable();
        let active_slabs = cache.active_slabs();
        let num_slabs = cache.num_slabs();
        let sharedavail = cache.total_depot_cached();

        let _ = writeln!(
            writer,
            "{:<17} {:6} {:6} {:7} {:10} {:12} : tunables {:7} {:10} {:12} : slabdata {:12} {:9} {:11}",
            cache.name(),
            active_objs,
            num_objs,
            obj_size,
            objs_per_slab,
            pages_per_slab,
            limit,
            batchcount,
            sharedfactor,
            active_slabs,
            num_slabs,
            sharedavail
        );
    }

    Ok(writer.len())
}
