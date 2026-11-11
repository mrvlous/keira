// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/vmstat` node reporting virtual memory and paging metrics.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted virtual memory counters into destination buffer.
pub fn read_vmstat(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    let (_total_bytes, used_bytes, free_bytes) = keira_mem::pmm::get_stats();
    let page_size = keira_mem::PAGE_SIZE;
    let nr_free_pages = free_bytes / page_size;
    let nr_alloc_pages = used_bytes / page_size;

    let (total_faults, cow_faults, stack_faults, demand_faults, _violations, tlb_flushes) =
        keira_mem::vmm_get_fault_stats();
    let (cow_active, cow_peak, cow_replicate, cow_inplace) = keira_mem::vmm_get_cow_stats();

    let mut nr_slab_reclaimable = 0usize;
    let mut nr_slab_unreclaimable = 0usize;
    for cache in keira_mem::get_all_kmem_caches() {
        nr_slab_reclaimable += cache.free_count() * cache.obj_size() / page_size as usize;
        nr_slab_unreclaimable += cache.allocated_count() * cache.obj_size() / page_size as usize;
    }

    let _ = write!(
        writer,
        "nr_free_pages {}\n\
         nr_alloc_pages {}\n\
         nr_page_table_pages 16\n\
         nr_dirty 0\n\
         nr_writeback 0\n\
         nr_slab_reclaimable {}\n\
         nr_slab_unreclaimable {}\n\
         pgpgin 0\n\
         pgpgout 0\n\
         pgfault {}\n\
         pgmajfault {}\n\
         pgfault_cow {}\n\
         pgfault_cow_inplace {}\n\
         pgfault_cow_replicate {}\n\
         nr_cow_active {}\n\
         nr_cow_peak {}\n\
         pgfault_stack {}\n\
         pgfault_demand {}\n\
         tlb_flushes {}\n",
        nr_free_pages,
        nr_alloc_pages,
        nr_slab_reclaimable,
        nr_slab_unreclaimable,
        total_faults,
        demand_faults,
        cow_faults,
        cow_inplace,
        cow_replicate,
        cow_active,
        cow_peak,
        stack_faults,
        demand_faults,
        tlb_flushes
    );

    Ok(writer.len())
}
