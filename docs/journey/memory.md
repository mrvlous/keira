<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Development Journey: Physical & Virtual Memory Architecture

This document chronicles the evolution of memory management in Keira Kernel from simple bitmap physical allocation to high-performance slab heaps and recursive page tables.

---

## Memory Evolution Flow

```mermaid
graph TD
    Phase1["1. Bitmap Frame Allocator (4KB Physical Pages)"] --> Phase2["2. 4-Level Paging with Higher-Half Virtual Mapping"]
    Phase2 --> Phase3["3. Slab & Buddy Kernel Heap Allocator"]
    Phase3 --> Phase4["4. DMA Continuous Buffers & Direct Physical Map"]
    Phase4 --> Phase5["5. Page Fault Demand Paging & KASLR"]
```

---

## Key Engineering Milestones

* **Zero-Allocation Bitmap PMM**: Bootstrapped physical memory allocator utilizing GRUB memory map tags to safely reserve kernel text and MMIO holes.
* **Recursive Page Table Navigation**: Implemented recursive PML4 mapping at slot 510, enabling dynamic mapping and unmapping of 4KB pages without extra page table allocations.
* **Slab Allocator**: Created power-of-two slab caches (32B to 4096B) to achieve sub-microsecond kernel allocations with minimal heap fragmentation.
