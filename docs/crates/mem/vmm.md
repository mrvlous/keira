<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Manager (VMM) & Paging

Documentation for 4-level paging and address spaces in [`crates/mem/src/vmm/`](../../../crates/mem/src/vmm).

## Virtual Address Space Layout

```
0x0000_0000_0000_0000 - 0x0000_003F_FFFF_FFFF : Identity Mapped Kernel (1GB)
0x0000_0040_0000_0000 - 0x0000_5FFF_FFFF_FFFF : Dynamic Kernel Modules & Heap
0x0000_6000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : Userland Address Space (ELF / brk / stack)
0x0000_7FFF_FE00_0000 - 0x0000_7FFF_FFFF_0000 : Ring 3 User Stack (16 Pages)
```

## Key APIs
- `clone_kernel_pml4() -> Result<u64, &'static str>`: Clones the kernel PML4 root table for isolated userland process tasks.
- `map_page(virt: u64, phys: u64, flags: u64) -> Result<(), &'static str>`: Maps a 4KB virtual page.
- `unmap_page(virt: u64) -> Result<(), &'static str>`: Unmaps a virtual page and invalidates TLB (`invlpg`).
- `get_phys_addr(virt: u64) -> Option<u64>`: Translates a virtual address across 4KB, 2MB, and 1GB huge page hierarchies to its exact physical address.
- `get_phys_addr_in_pml4(pml4_phys: u64, virt: u64) -> Option<u64>`: Translates a virtual address within a specific PML4 root table.
- `get_pte_in_pml4(pml4_phys: u64, virt: u64) -> Option<u64>`: Retrieves the raw page table entry (PTE, 2MB PDE, or 1GB PDPTE).
- `translate_pte_to_phys(pte: u64, virt: u64, level: u8) -> u64`: Reconstructs physical addresses using level-specific masks (`PTE_ADDR_MASK_4K`, `PTE_ADDR_MASK_2M`, `PTE_ADDR_MASK_1G`).
- `free_and_unmap_page(virt: u64) -> Result<(), &'static str>`: Unmaps a virtual page and safely reclaims underlying 4KB frames while preserving huge page structures.
- `free_user_pages(pml4_phys: u64, program_break: u64)`: Reclaims all user-space page table trees, process frames, and VMA metadata while skipping static huge pages.
- `sys_mmap(hint: u64, len: u64, prot: u32, flags: u32) -> Result<u64, &'static str>`: Allocates and maps anonymous virtual memory regions with collision checks against active VMAs and present 4KB/2MB/1GB page table entries.
- `sys_munmap(addr: u64, len: u64) -> Result<(), &'static str>`: Unmaps user virtual memory regions and synchronizes VMA metadata.
- `sys_munmap_ext(addr: u64, len: u64) -> Result<u64, (&'static str, u64)>`: Unmaps memory ranges and exposes the exact byte count successfully unmapped.
- `sys_mprotect(addr: u64, len: u64, prot: u32) -> Result<(), &'static str>`: Modifies memory protection bits and splits VMA ranges.
- `verify_vma_pte_invariants(pml4_phys: u64) -> Result<(), &'static str>`: Verifies bi-directional synchronization between VMA descriptors and hardware PTE attributes, detecting orphan page mappings.
- `switch_address_space(pml4_phys: u64)`: Writes the physical PML4 table address to `CR3`.
