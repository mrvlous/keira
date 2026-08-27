<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 2: Frame Allocator & 4-Level Virtual Paging

This journal entry details the construction of the physical memory manager, 4-level virtual paging, and heap allocators in Keira Kernel.

---

## Engineering Challenges

1. **Bootstrap Memory Paradox**: You cannot allocate dynamic memory to track available memory before the memory allocator exists.
2. **Page Table Recursion**: Mapping virtual address ranges requires modifying page tables, which themselves exist at physical memory addresses.

---

## Solutions & Design Choices

* **Static Bitmap PMM**: Placed a static bit array in the kernel BSS segment to track 4096-byte frames without requiring any dynamic heap allocation.
* **4-Level Paging Engine**: Built an explicit page table walker allocating PDPT, PD, and PT frames on demand from the PMM, enforcing `W^X` (Write XOR Execute) memory protections.
* **Thread-Safe Slab Heap**: Implemented a 16-byte aligned bump and slab heap allocator (`kmalloc` / `kfree`) backed by atomic CAS operations.
