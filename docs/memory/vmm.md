<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Manager (VMM) & Paging

This document describes the 4-level paging architecture (`x86_64`) and two-level paging (`i686`) used in Keira Kernel.

---

## 4-Level Paging Layout (`x86_64`)

Virtual addresses are decomposed into four 9-bit table indices:

```
63        48 47    39 38    30 29    21 20    12 11          0
+-----------+--------+--------+--------+--------+------------+
| Sign Ext. | PML4   | PDPT   | PD     | PT     | Page Offset|
| (16 bits) |(9 bits)|(9 bits)|(9 bits)|(9 bits)| (12 bits)  |
+-----------+--------+--------+--------+--------+------------+
```

---

## Page Table Flags

| Flag Bit | Constant | Description |
| :--- | :--- | :--- |
| `0` | `PAGE_PRESENT` | Page resides in physical memory (`1 = Present`) |
| `1` | `PAGE_WRITABLE` | Page is writable (`0 = Read-Only`) |
| `2` | `PAGE_USER` | Page accessible in User Mode Ring 3 (`DPL=3`) |
| `3` | `PAGE_WRITE_THROUGH` | Write-through caching policy |
| `4` | `PAGE_CACHE_DISABLE` | Disable CPU caching (for MMIO regions) |
| `9` | `PAGE_COW` | Copy-on-Write software flag (Bit 9 available for kernel use) |
| `63` | `PAGE_NO_EXECUTE` | Hardware `NX` bit preventing code execution |

---

## Process Address Space Duplication & Memory Isolation (`sys_fork`)

Keira implements process memory virtualization during task forking:

1. **Address Space Duplication**: During `sys_fork()`, `clone_user_address_space` allocates a new PML4 and PDPT root hierarchy, preserving kernel identity-map and MMIO regions while duplicating the userland address space.
2. **Dedicated Physical Frame Isolation**: For every mapped userland page, the VMM allocates a distinct, dedicated physical frame from the PMM and deep-copies the 4096-byte contents from the parent frame to the child frame.
3. **Safe Memory Reclamation**: Because parent and child hold independent physical frames, process termination and memory reaping in `sys_waitpid()` via `free_user_pages()` releases only the exiting task's private frames, leaving the remaining processes unaffected with zero dangling pointers or page-table corruption.
4. **Resumed Execution**: Ring 3 execution resumes transparently in both parent and child with completely isolated, independently mutable memory address spaces.

---

## Demand Paging & Lazy Allocation Architecture

Keira implements demand paging and lazy memory allocation to minimize physical frame consumption and eliminate eager allocation latency:

1. **Anonymous Memory Mapping (`sys_mmap`)**:
   - When userland requests anonymous memory via `sys_mmap` without the `MAP_POPULATE` (`0x08000`) flag, the kernel registers the Virtual Memory Area (VMA) but defers physical frame allocation.
   - Physical page allocation occurs lazily on first access when the CPU raises a Page Fault (`#PF`).
   - If `MAP_POPULATE` is passed, the kernel eagerly allocates and maps all physical frames immediately.
2. **File-Backed Memory Mapping & Disk Synchronization (`sys_mmap` / `sys_msync`)**:
   - For file-backed mappings, `sys_mmap` records the canonical VFS file path, file offset, and initial file size in the allocated VMA.
   - Initial read/write accesses generate a `#PF`. The fault handler allocates a physical frame from the PMM, computes the page offset into the file, and reads storage blocks into memory via decoupled kernel file hooks.
   - In shared mappings (`MAP_SHARED`), memory mutations mark the hardware dirty bit (`PAGE_DIRTY`).
   - Calling `sys_msync()` walks the active VMA's page table entries, writes modified frames back to FAT16 disk storage using `write_file_offset`, and clears the dirty bit.
3. **Process Heap Auto-Expansion (`sys_brk` / `sbrk`)**:
   - Calling `sys_brk()` or userland `sbrk()` increments the task's `program_break` pointer without pre-allocating physical memory.
   - Accessing newly extended heap addresses triggers an Interrupt 14 (`#PF`) fault with `CR2` falling between `program_break_start` and `program_break`.
   - The kernel page fault handler validates the bounds, allocates a zeroed physical frame from the PMM, maps the page with User and Writable permissions (`PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER`), and invalidates the CPU TLB via `invlpg`.
   - Shrinking the heap via `sys_brk()` unmaps and frees all physical frames above the new break boundary.
4. **User Stack Auto-Growth**:
   - When a Ring 3 user process accesses an unmapped virtual address within its authorized user stack window (`USER_STACK_BOTTOM` to `USER_STACK_TOP`), the CPU triggers `#PF`.
   - The handler verifies stack limits, maps an on-demand zeroed page, and resumes execution seamlessly.
5. **Lazy `munmap` and PTE Invariant Verification**:
   - When releasing memory regions (`sys_munmap`), the VMM inspects the page directory hierarchy using `is_page_mapped_in_pml4`. Unaccessed lazy pages without allocated physical frames are safely bypassed, preventing double-free panics in the PMM allocator.
   - For file-backed VMAs, trimming or partial unmapping updates `file_offset` and active boundaries while leaving the underlying storage intact.

---

## 2MB Huge Pages (`PAGE_HUGE`)

To optimize memory bandwidth and reduce TLB miss overhead for massive contiguous allocations (such as the linear VBE/GOP framebuffer and kernel direct physical memory mappings), Keira supports 2MB Huge Pages directly in Level 2 Page Directories (PD):

- **Page Size Flag (`PAGE_HUGE = 1 << 7`)**: Set in the Page Directory Entry (PDE).
- **Physical Address Alignment**: Both virtual address and physical frame are aligned to 2MB boundaries (`0x20_0000`).
- **Core API (`crates/mem/src/vmm/paging.rs`)**:
  - `map_huge_2m_page(vaddr, paddr, flags)`: Creates a direct 2MB translation bypassing the 4KB PT level.
  - `unmap_huge_2m_page(vaddr)`: Clears the huge page entry and invalidates the CPU TLB via `invlpg`.
- **TLB Advantage**: A single 2MB PDE translation entry covers 512 regular 4KB pages, cutting TLB pressure by a factor of 512.
