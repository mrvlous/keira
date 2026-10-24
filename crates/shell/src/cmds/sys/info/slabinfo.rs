// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'slabinfo' shell command.

use crate::args::CliArgs;
use keira_io::vga;
use keira_mem::MAX_CPU_CORES;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: slabinfo [-c] [-v] [-h]\n\n");
        vga::print_str(
            "Description:\n  Display slab object caches, magazine utilization and memory allocation telemetry.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str(
            "  -c, --cpu      Display per-CPU magazine caches, hits and depot exchanges\n",
        );
        vga::print_str("  -v, --verbose  Display comprehensive slab tunables, depot cache and memory footprints\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    if args.has_flag('c', "cpu") {
        display_per_cpu_depots();
        return;
    }

    if args.has_flag('v', "verbose") {
        display_verbose_slabinfo();
        return;
    }

    display_standard_slabinfo();
}

fn display_standard_slabinfo() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Kernel Slab Object Caches (slabinfo 2.1):\n");
    vga::print_str(
        "CACHE NAME        SIZE   ACTIVE  TOTAL   SLABS   MEM(KB)  ALLOC_HIT  FREE_HIT\n",
    );
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(
        "----------------  -----  ------  ------  ------  -------  ---------  --------\n",
    );

    let caches = keira_mem::get_all_kmem_caches();
    let mut total_active_objs = 0usize;
    let mut total_objs = 0usize;
    let mut total_mem_bytes = 0usize;

    for cache in caches.iter() {
        let active = cache.allocated_count();
        let total = cache.total_count();
        let obj_size = cache.obj_size();
        let slabs = cache.num_slabs();
        let mem_kb = (total * obj_size) / 1024;

        total_active_objs += active;
        total_objs += total;
        total_mem_bytes += total * obj_size;

        vga::set_color(vga::Color::White, vga::Color::Black);
        print_fixed_str(cache.name(), 18);
        print_fixed_u64(obj_size as u64, 7);
        print_fixed_u64(active as u64, 8);
        print_fixed_u64(total as u64, 8);
        print_fixed_u64(slabs as u64, 8);
        print_fixed_u64(mem_kb as u64, 9);
        print_fixed_u64(cache.total_alloc_hits() as u64, 11);
        print_fixed_u64(cache.total_free_hits() as u64, 8);
        vga::print_str("\n");
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(
        "-----------------------------------------------------------------------------\n",
    );
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Summary: ");
    vga::print_u64(total_active_objs as u64);
    vga::print_str(" active objects, ");
    vga::print_u64(total_objs.saturating_sub(total_active_objs) as u64);
    vga::print_str(" cached objects, ");
    vga::print_u64((total_mem_bytes / 1024) as u64);
    vga::print_str(" KB allocated memory\n");
}

fn display_per_cpu_depots() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Per-CPU Magazine Depot Telemetry:\n");
    vga::print_str("CACHE NAME        CORE  CACHED  ALLOC_HIT  FREE_HIT  EXCHANGES\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("----------------  ----  ------  ---------  --------  ---------\n");

    let caches = keira_mem::get_all_kmem_caches();
    for cache in caches.iter() {
        for core in 0..MAX_CPU_CORES {
            if let Some((cached, alloc_hits, free_hits, exchanges)) = cache.cpu_stats(core) {
                if cached > 0 || alloc_hits > 0 || free_hits > 0 || exchanges > 0 || core == 0 {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    print_fixed_str(cache.name(), 18);
                    print_fixed_u64(core as u64, 6);
                    print_fixed_u64(cached as u64, 8);
                    print_fixed_u64(alloc_hits as u64, 11);
                    print_fixed_u64(free_hits as u64, 10);
                    print_fixed_u64(exchanges as u64, 9);
                    vga::print_str("\n");
                }
            }
        }
    }
}

fn display_verbose_slabinfo() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Kernel Slab Allocator Comprehensive Telemetry:\n");
    vga::print_str(
        "CACHE NAME        ACTIVE/TOTAL  OBJ/SLAB  PG/SLAB  TUNABLES      DEPOT_CACHED\n",
    );
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(
        "----------------  ------------  --------  -------  ------------  ------------\n",
    );

    let caches = keira_mem::get_all_kmem_caches();
    for cache in caches.iter() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        print_fixed_str(cache.name(), 18);

        let active = cache.allocated_count();
        let total = cache.total_count();
        let mut count_buf = [b' '; 14];
        let mut offset = format_two_u64(active as u64, total as u64, &mut count_buf);
        while offset < 14 {
            count_buf[offset] = b' ';
            offset += 1;
        }
        if let Ok(s) = core::str::from_utf8(&count_buf[..offset]) {
            vga::print_str(s);
        }

        print_fixed_u64(cache.objs_per_slab() as u64, 10);
        print_fixed_u64(cache.pages_per_slab() as u64, 9);

        let mut tune_buf = [b' '; 14];
        let mut t_off = format_tunables(
            cache.limit_tunable() as u64,
            cache.batchcount_tunable() as u64,
            cache.sharedfactor_tunable() as u64,
            &mut tune_buf,
        );
        while t_off < 14 {
            tune_buf[t_off] = b' ';
            t_off += 1;
        }
        if let Ok(s) = core::str::from_utf8(&tune_buf[..t_off]) {
            vga::print_str(s);
        }

        print_fixed_u64(cache.total_depot_cached() as u64, 12);
        vga::print_str("\n");
    }
}

fn print_fixed_str(s: &str, width: usize) {
    vga::print_str(s);
    for _ in 0..width.saturating_sub(s.len()) {
        vga::print_str(" ");
    }
}

fn print_fixed_u64(val: u64, width: usize) {
    let mut len = 0;
    let mut temp = val;
    if temp == 0 {
        len = 1;
    } else {
        while temp > 0 {
            len += 1;
            temp /= 10;
        }
    }
    vga::print_u64(val);
    for _ in 0..width.saturating_sub(len) {
        vga::print_str(" ");
    }
}

fn format_two_u64(a: u64, b: u64, buf: &mut [u8]) -> usize {
    let mut writer = FormatWriter::new(buf);
    use core::fmt::Write;
    let _ = write!(writer, "{}/{}", a, b);
    writer.len
}

fn format_tunables(limit: u64, batch: u64, shared: u64, buf: &mut [u8]) -> usize {
    let mut writer = FormatWriter::new(buf);
    use core::fmt::Write;
    let _ = write!(writer, "{}/{}/{}", limit, batch, shared);
    writer.len
}

struct FormatWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FormatWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
    }
}

impl<'a> core::fmt::Write for FormatWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let avail = self.buf.len().saturating_sub(self.len);
        let count = bytes.len().min(avail);
        self.buf[self.len..self.len + count].copy_from_slice(&bytes[..count]);
        self.len += count;
        Ok(())
    }
}
