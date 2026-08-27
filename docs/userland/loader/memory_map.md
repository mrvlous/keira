<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Segment Memory Mapping & Virtual Protection

This document specifies virtual page allocation, zero-filling BSS, and `W^X` memory protection during ELF loading.

---

## Segment Mapping Process

1. **Address Space Allocation**: Clones kernel page tables to produce an isolated user PML4.
2. **Page Allocation**: Allocates 4KB frames from PMM for each segment page.
3. **Copy & Zero-Fill**: Copies `p_filesz` bytes from the file into the allocated frames, zero-filling remaining bytes up to `p_memsz` (BSS section).
4. **Flag Translation**:
   - `PF_R`: Sets `PAGE_PRESENT`.
   - `PF_W`: Sets `PAGE_WRITABLE`.
   - `PF_X`: Clears `PAGE_NO_EXECUTE`.
5. **User Bit**: Sets `PAGE_USER` bit on all allocated segment pages.
