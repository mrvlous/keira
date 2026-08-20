<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Memory Model & Virtual Paging

Keira Kernel utilizes x86_64 4-level hierarchical paging (PML4 -> PDPT -> PD -> PT) with 4KB page frames.

## Address Space Segmentation

```
+-------------------------------------------------------------+ 0xFFFF_FFFF_FFFF_FFFF
| Top of Virtual Address Space (Reserved)                    |
+-------------------------------------------------------------+ 0x0000_7FFF_FFFF_FFFF
| Ring 3 Userland Process Address Space                       |
|   - 0x0000_7FFF_FE00_0000 : User Stack Top (16 Pages)      |
|   - 0x0000_6000_0000_0000 : User Program Break (brk Heap)  |
|   - 0x0000_0040_0000_0000 : ELF Code (.text) & Data (.data) |
+-------------------------------------------------------------+ 0x0000_0040_0000_0000
| Dynamic Kernel Modules & Shared IPC Memory                  |
+-------------------------------------------------------------+ 0x0000_0000_4000_0000 (1GB)
| Ring 0 Identity-Mapped Kernel Space (0 - 1GB)               |
|   - 0x0000_0000_FD00_0000 : VBE Linear Framebuffer (MMIO)   |
|   - 0x0000_0000_0020_0000 : Kernel Dynamic Heap (1MB+)      |
|   - 0x0000_0000_0010_0000 : Kernel Code & BSS (1MB)         |
|   - 0x0000_0000_000B_8000 : VGA Text Buffer (80x25)        |
+-------------------------------------------------------------+ 0x0000_0000_0000_0000
```

## Address Space Cloning
When launching userland programs (`run /apps/bin/gcc.elf`), `vmm::clone_kernel_pml4()` creates an isolated PML4 table that retains kernel identity mappings in `PDPT[0]` and framebuffer MMIO in `PDPT[3]`, but provides dedicated, isolated user pages.
