<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Memory Model & Virtual Paging

Keira Kernel implements architecture-tailored memory models across both 64-bit Long Mode (`x86_64`) and 32-bit Protected Mode (`i686`).

---

## 1. 64-bit Virtual Paging Model (`x86_64`)

Utilizes standard x86_64 4-level hierarchical paging (`PML4` -> `PDPT` -> `PD` -> `PT`) with 4KB page frames:

```text
+-------------------------------------------------------------+ 0xFFFF_FFFF_FFFF_FFFF
| Top of Virtual Address Space (Reserved)                    |
+-------------------------------------------------------------+ 0x0000_7FFF_FFFF_FFFF
| Ring 3 Userland Process Address Space                       |
|   - 0x0000_7FFF_FE00_0000 : User Stack Top (16 Pages)      |
|   - 0x0000_6000_0000_0000 : User Program Break (brk Heap)  |
|   - 0x0000_0000_4000_0000 : User ELF Base (.text / .data)  |
+-------------------------------------------------------------+ 0x0000_0000_4000_0000 (1GB)
| Ring 0 Identity-Mapped Kernel Space (0 - 1GB)               |
|   - 0x0000_0000_FD00_0000 : VBE Linear Framebuffer (MMIO)   |
|   - 0x0000_0000_0020_0000 : Kernel Dynamic Heap (1MB+)      |
|   - 0x0000_0000_0010_0000 : Kernel Code & BSS (1MB)         |
|   - 0x0000_0000_000B_8000 : VGA Text Buffer (80x25)        |
+-------------------------------------------------------------+ 0x0000_0000_0000_0000
```

### Address Space Cloning
When launching userland programs (`run /system/bin/kcc.elf`), `vmm::clone_kernel_pml4()` creates an isolated child PML4 table that retains kernel identity mappings in `PDPT[0]` and framebuffer MMIO in `PDPT[3]`, while giving user space dedicated, isolated pages at `0x40000000`.

---

## 2. 32-bit Protected Mode Model (`i686`)

In 32-bit Protected Mode, Keira operates with flat segmentation and hardware-enforced Ring 3 privileges:
* **Kernel Memory (0 - 16 MiB)**: Code, heap, stack, and VGA buffers.
* **Userland Binary Base**: `0x01000000` (16 MiB boundary linked via `user/linker32.ld`).
* **Userland Stack Top**: `0x07FFF000 - 16` (~128 MiB boundary within standard physical bounds).
* **Segment Selectors**: User Code `CS=0x1B` (GDT index 3, RPL 3) and User Data `DS/SS=0x23` (GDT index 4, RPL 3).
