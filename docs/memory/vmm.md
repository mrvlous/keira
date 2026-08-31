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
| `63` | `PAGE_NO_EXECUTE` | Hardware `NX` bit preventing code execution |

---

## Demand Paging & User Stack Auto-Growth

When a Ring 3 user process accesses an unmapped virtual address within its authorized Virtual Memory Area (VMA) or user stack window (`USER_STACK_BOTTOM` to `USER_STACK_TOP`), the CPU triggers Interrupt 14 (`#PF` Page Fault):

1. **Hardware Fault Trapping**: The CPU writes the faulting address to `CR2` and pushes an error code containing fault attributes (`P`, `W/R`, `U/S`, `I/D`).
2. **Exception Dispatcher (`handle_page_fault`)**: The Ring 0 handler inspects `CR2` and validates against active VMAs or the dynamic user stack window.
3. **On-Demand Frame Allocation**: If the access is valid and the page is not-present, a zeroed physical frame is allocated from PMM, mapped to the faulting page, and TLB entry is invalidated via `invlpg`.
4. **Transparent Instruction Resume**: The interrupt handler returns via `iretq`, allowing the CPU to resume execution seamlessly without process crashes or memory leaks.
