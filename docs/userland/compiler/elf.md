<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC ELF Binary Generation (`userland/bin/kcc/elf/`)

Directly produces valid executable ELF files without requiring external assemblers or linkers.

---

## Emitted ELF Structure

* **ELF Header (`Elf32_Ehdr`)**: Magic bytes `ELF`, machine type `EM_386`, entry address pointing to generated code.
* **Program Headers (`Elf32_Phdr`)**:
  * Segment 1: `PT_LOAD`, flags `PF_R | PF_X` (Text and read-only data).
  * Segment 2: `PT_LOAD`, flags `PF_R | PF_W` (Writable data and BSS).
* **Section Headers (`Elf32_Shdr`)**: `.text`, `.data`, `.rodata`, `.bss`, `.symtab`, `.strtab`, `.shstrtab`.
