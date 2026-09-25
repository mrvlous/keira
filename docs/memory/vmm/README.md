<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Manager (VMM)

The VMM manages address space isolation, page tables, memory mappings, and demand paging.

---

## VMM Index

| Document | Description |
| :--- | :--- |
| [`paging_tables.md`](paging_tables.md) | Multi-level page table structures (`PML4`, `PDPT`, `PD`, `PT`) |
| [`areas.md`](areas.md) | Virtual Memory Areas (VMAs), mmap, and protection flags |
| [`fault_reclaim.md`](fault_reclaim.md) | Page fault handler (`#PF`), demand paging, and page reclamation |
