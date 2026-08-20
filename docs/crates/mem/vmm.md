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
- `switch_address_space(pml4_phys: u64)`: Writes the physical PML4 table address to `CR3`.
