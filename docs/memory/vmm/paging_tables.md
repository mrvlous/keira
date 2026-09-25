<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Manager (VMM) & 4-Level Paging

Keira manages virtual memory translation through hardware page tables, supporting 4-level paging on `x86_64` and 2-level paging on `i686`.

---

## 1. x86_64 4-Level Paging Structure

A 48-bit virtual address translates through four hierarchical tables loaded into register `CR3`:

```text
 47        39 38        30 29        21 20        12 11          0
+------------+------------+------------+------------+-------------+
| PML4 Index | PDPT Index |  PD Index  |  PT Index  | Page Offset |
|   (9 bits) |   (9 bits) |   (9 bits) |   (9 bits) |   (12 bits) |
+------------+------------+------------+------------+-------------+
```

1. **PML4** (Page Map Level 4): Points to Page Directory Pointer Table (PDPT).
2. **PDPT** (Page Directory Pointer Table): Points to Page Directory (PD).
3. **PD** (Page Directory): Points to Page Table (PT).
4. **PT** (Page Table): Points to 4 KiB Physical Frame.

---

## 2. Page Table Entry (PTE) Flags

Each entry in all page table levels is 64 bits wide:

| Bit | Name | Description |
| :--- | :--- | :--- |
| `0` | **P (Present)** | Must be `1` for valid translation; triggers `#PF` if `0` |
| `1` | **R/W (Read/Write)** | `0` = Read-only; `1` = Read and Write |
| `2` | **U/S (User/Supervisor)** | `0` = Ring 0 only; `1` = Userland Ring 3 accessible |
| `3` | **PWT (Write-Through)** | `1` = Write-through caching; `0` = Write-back caching |
| `4` | **PCD (Cache Disable)** | `1` = Cache disabled (used for MMIO device registers) |
| `5` | **A (Accessed)** | Set by hardware when page is read or executed |
| `6` | **D (Dirty)** | Set by hardware when page is written |
| `7` | **PS (Page Size)** | `1` = Huge page (2 MiB at PD level, 1 GiB at PDPT level) |
| `8` | **G (Global)** | Prevents TLB flush on `CR3` reload |
| `63` | **NX (No-Execute)** | `1` = Instruction fetches forbidden (enforces W^X security) |
