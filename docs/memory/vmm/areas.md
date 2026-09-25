<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Virtual Memory Areas (VMAs) & Memory Mapping

Each process address space maintains an organized collection of Virtual Memory Areas (`crates/mem/src/vmm/area/`).

---

## VMA Attributes

* Range: `[start_address, end_address)` page-aligned.
* Flags: `PROT_READ`, `PROT_WRITE`, `PROT_EXEC`.
* Backing: Anonymous RAM, file-backed (memory-mapped files), or memory-mapped device I/O.
