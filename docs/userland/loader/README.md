<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Dynamic ELF64 Binary Loader

This submodule details the 64-bit Executable and Linkable Format (ELF) binary loader in Keira Kernel.

---

## Loading Pipeline

```mermaid
graph TD
    File["ELF Binary on VFS (/system/bin/*.elf)"] --> Header["elf_format.md<br/>Verify ELF Magic & Header"]
    Header --> Segments["memory_map.md<br/>Map PT_LOAD Segments into VMM"]
    Segments --> Stack["Allocate 64 KB User Stack"]
    Stack --> Switch["Drop to Ring 3 & Jump to e_entry"]
```

---

## Loader Submodule Index

| Document | Topic | Description |
| :--- | :--- | :--- |
| [`elf_format.md`](elf_format.md) | ELF Header Specifications | ELF magic (`0x7F 'E' 'L' 'F'`), 64-bit header fields, and program headers |
| [`memory_map.md`](memory_map.md) | Segment Memory Mapping | Virtual memory mapping of `PT_LOAD` segments with `R/W/X` permission bits |
