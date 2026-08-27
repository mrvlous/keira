<!-- SPDX-License-Identifier: GPL-2.0-only -->

# 64-Bit ELF Format & Header Verification

This document specifies ELF64 header validation and program header table parsing in Keira Kernel.

---

## 64-Bit ELF Header Structure

```rust
#[repr(C, packed)]
pub struct Elf64Header {
    pub e_ident: [u8; 16], // Magic: 0x7F 'E' 'L' 'F', Class 2 (64-bit), LSB
    pub e_type: u16,       // 2 = ET_EXEC (Executable)
    pub e_machine: u16,    // 0x3E = EM_X86_64
    pub e_version: u32,    // 1 = EV_CURRENT
    pub e_entry: u64,      // Program Entry Virtual Address
    pub e_phoff: u64,      // Program Header Table File Offset
    pub e_shoff: u64,      // Section Header Table File Offset
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,      // Number of Program Header entries
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}
```

---

## Program Header Structure (`PT_LOAD`)

```rust
#[repr(C, packed)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,       // 1 = PT_LOAD
    pub p_flags: u32,      // PF_X (1), PF_W (2), PF_R (4)
    pub p_offset: u64,     // Segment file offset
    pub p_vaddr: u64,      // Segment virtual memory base address
    pub p_paddr: u64,
    pub p_filesz: u64,     // Segment byte length in file
    pub p_memsz: u64,      // Segment byte length in memory (memsz >= filesz)
    pub p_align: u64,      // Page alignment (4096)
}
```
