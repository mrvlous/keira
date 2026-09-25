<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 2: Memory Management & Virtual Paging

This milestone details the design and implementation of physical frame allocation, virtual memory paging, and dynamic kernel heap management.

---

## Key Achievements

1. **Physical Memory Manager (PMM)**: Bitmapped frame allocator managing 4 KiB physical pages with lock-free synchronization.
2. **Virtual Memory Manager (VMM)**: Recursive 4-level paging (`PML4 -> PDPT -> PD -> PT`) on `x86_64` and 2-level paging on `i686`.
3. **Segregated Free-List Heap**: Kernel heap allocator with 9 size classes, memory boundary canaries, and zero-fragmentation pool management.
4. **Swap Paging Engine**: Disk-backed swap partition supporting page-out and on-demand page fault resolution.
