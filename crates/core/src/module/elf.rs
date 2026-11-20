// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Freestanding ELF relocatable object (`ET_REL`) parser.
//!
//! Provides zero-allocation validation and section/symbol extraction for
//! 64-bit (`x86_64`) and 32-bit (`i686`) kernel loadable object files.

use crate::error::KernelError;

/// ELF magic header constant bytes `\x7fELF`.
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF class identifying target bitness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass {
    /// 32-bit architecture (`ELFCLASS32`).
    Elf32 = 1,
    /// 64-bit architecture (`ELFCLASS64`).
    Elf64 = 2,
}

/// Relocatable ELF file type constant (`ET_REL`).
pub const ET_REL: u16 = 1;

/// x86_64 machine type constant (`EM_X86_64`).
pub const EM_X86_64: u16 = 62;

/// Intel 80386 32-bit machine type constant (`EM_386`).
pub const EM_386: u16 = 3;

/// Section header type constants.
pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_NOBITS: u32 = 8;
pub const SHT_REL: u32 = 9;

/// Section attribute flags.
pub const SHF_WRITE: u64 = 0x1;
pub const SHF_ALLOC: u64 = 0x2;
pub const SHF_EXECINSTR: u64 = 0x4;

/// Symbol binding constants.
pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;

/// Symbol type constants.
pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const STT_SECTION: u8 = 3;

/// Undefined section index indicating external reference.
pub const SHN_UNDEF: u16 = 0;

/// Parsed ELF file header information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfHeaderInfo {
    /// Architecture class (32-bit or 64-bit).
    pub class: ElfClass,
    /// Machine architecture type.
    pub machine: u16,
    /// Offset of section header table in bytes.
    pub shoff: u64,
    /// Size of each section header entry in bytes.
    pub shentsize: usize,
    /// Number of section header entries.
    pub shnum: usize,
    /// Section header table index of the section name string table.
    pub shstrndx: usize,
}

/// Section header descriptor extracted from ELF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfSectionHeader {
    /// Section table index.
    pub index: usize,
    /// Byte offset in section name string table (`.shstrtab`).
    pub name_offset: u32,
    /// Section type (e.g. `SHT_PROGBITS`, `SHT_RELA`).
    pub sh_type: u32,
    /// Section memory and execution attributes.
    pub sh_flags: u64,
    /// Virtual address where section resides in memory.
    pub sh_addr: u64,
    /// Byte offset of section contents in raw file.
    pub sh_offset: u64,
    /// Section size in bytes.
    pub sh_size: u64,
    /// Section header table link index.
    pub sh_link: u32,
    /// Extra section information or relocation target section index.
    pub sh_info: u32,
    /// Address alignment constraints.
    pub sh_addralign: u64,
    /// Fixed entry size in bytes for table sections.
    pub sh_entsize: u64,
}

/// Symbol entry extracted from `.symtab`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfSymbolInfo {
    /// Byte offset of symbol identifier in string table (`.strtab`).
    pub name_offset: u32,
    /// Symbol value or section-relative offset.
    pub value: u64,
    /// Associated symbol size in bytes.
    pub size: u64,
    /// Associated section header table index (`SHN_UNDEF` if external).
    pub shndx: u16,
    /// Symbol type (`STT_FUNC`, `STT_OBJECT`, etc.).
    pub sym_type: u8,
    /// Symbol binding scope (`STB_LOCAL`, `STB_GLOBAL`, `STB_WEAK`).
    pub sym_bind: u8,
}

/// Relocation record extracted from `.rel` or `.rela` sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfRelocInfo {
    /// Offset within the target section where relocation must be applied.
    pub offset: u64,
    /// Symbol table index referenced by this relocation record.
    pub sym_idx: usize,
    /// Architecture-specific relocation type.
    pub rel_type: u32,
    /// Explicit addend value for `RELA` (0 for `REL`).
    pub addend: i64,
}

/// Helper function to read a 16-bit little-endian integer.
pub fn read_u16_le(data: &[u8], offset: usize) -> Result<u16, KernelError> {
    if offset + 2 > data.len() {
        return Err(KernelError::Fault);
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

/// Helper function to read a 32-bit little-endian integer.
pub fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, KernelError> {
    if offset + 4 > data.len() {
        return Err(KernelError::Fault);
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Helper function to read a 64-bit little-endian integer.
pub fn read_u64_le(data: &[u8], offset: usize) -> Result<u64, KernelError> {
    if offset + 8 > data.len() {
        return Err(KernelError::Fault);
    }
    Ok(u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]))
}

/// Parses and validates the ELF header from raw binary bytes.
pub fn parse_elf_header(data: &[u8]) -> Result<ElfHeaderInfo, KernelError> {
    if data.len() < 16 {
        return Err(KernelError::InvalidArgument);
    }

    if data[0..4] != ELF_MAGIC {
        return Err(KernelError::InvalidArgument);
    }

    let class = match data[4] {
        1 => ElfClass::Elf32,
        2 => ElfClass::Elf64,
        _ => return Err(KernelError::InvalidArgument),
    };

    // Verify 2's complement little-endian
    if data[5] != 1 {
        return Err(KernelError::NotSupported);
    }

    // Verify EV_CURRENT (1)
    if data[6] != 1 {
        return Err(KernelError::InvalidArgument);
    }

    match class {
        ElfClass::Elf64 => {
            if data.len() < 64 {
                return Err(KernelError::InvalidArgument);
            }
            let e_type = read_u16_le(data, 16)?;
            if e_type != ET_REL {
                return Err(KernelError::InvalidArgument);
            }
            let machine = read_u16_le(data, 18)?;
            let shoff = read_u64_le(data, 40)?;
            let shentsize = read_u16_le(data, 58)? as usize;
            let shnum = read_u16_le(data, 60)? as usize;
            let shstrndx = read_u16_le(data, 62)? as usize;

            Ok(ElfHeaderInfo {
                class,
                machine,
                shoff,
                shentsize,
                shnum,
                shstrndx,
            })
        }
        ElfClass::Elf32 => {
            if data.len() < 52 {
                return Err(KernelError::InvalidArgument);
            }
            let e_type = read_u16_le(data, 16)?;
            if e_type != ET_REL {
                return Err(KernelError::InvalidArgument);
            }
            let machine = read_u16_le(data, 18)?;
            let shoff = read_u32_le(data, 32)? as u64;
            let shentsize = read_u16_le(data, 46)? as usize;
            let shnum = read_u16_le(data, 48)? as usize;
            let shstrndx = read_u16_le(data, 50)? as usize;

            Ok(ElfHeaderInfo {
                class,
                machine,
                shoff,
                shentsize,
                shnum,
                shstrndx,
            })
        }
    }
}

/// Parses a specific section header by table index.
pub fn parse_section_header(
    data: &[u8],
    hdr: &ElfHeaderInfo,
    index: usize,
) -> Result<ElfSectionHeader, KernelError> {
    if index >= hdr.shnum {
        return Err(KernelError::NotFound);
    }

    let offset = (hdr.shoff as usize) + (index * hdr.shentsize);
    if offset + hdr.shentsize > data.len() {
        return Err(KernelError::Fault);
    }

    match hdr.class {
        ElfClass::Elf64 => {
            if hdr.shentsize < 64 {
                return Err(KernelError::InvalidArgument);
            }
            let name_offset = read_u32_le(data, offset)?;
            let sh_type = read_u32_le(data, offset + 4)?;
            let sh_flags = read_u64_le(data, offset + 8)?;
            let sh_addr = read_u64_le(data, offset + 16)?;
            let sh_offset = read_u64_le(data, offset + 24)?;
            let sh_size = read_u64_le(data, offset + 32)?;
            let sh_link = read_u32_le(data, offset + 40)?;
            let sh_info = read_u32_le(data, offset + 44)?;
            let sh_addralign = read_u64_le(data, offset + 48)?;
            let sh_entsize = read_u64_le(data, offset + 56)?;

            Ok(ElfSectionHeader {
                index,
                name_offset,
                sh_type,
                sh_flags,
                sh_addr,
                sh_offset,
                sh_size,
                sh_link,
                sh_info,
                sh_addralign,
                sh_entsize,
            })
        }
        ElfClass::Elf32 => {
            if hdr.shentsize < 40 {
                return Err(KernelError::InvalidArgument);
            }
            let name_offset = read_u32_le(data, offset)?;
            let sh_type = read_u32_le(data, offset + 4)?;
            let sh_flags = read_u32_le(data, offset + 8)? as u64;
            let sh_addr = read_u32_le(data, offset + 12)? as u64;
            let sh_offset = read_u32_le(data, offset + 16)? as u64;
            let sh_size = read_u32_le(data, offset + 20)? as u64;
            let sh_link = read_u32_le(data, offset + 24)?;
            let sh_info = read_u32_le(data, offset + 28)?;
            let sh_addralign = read_u32_le(data, offset + 32)? as u64;
            let sh_entsize = read_u32_le(data, offset + 36)? as u64;

            Ok(ElfSectionHeader {
                index,
                name_offset,
                sh_type,
                sh_flags,
                sh_addr,
                sh_offset,
                sh_size,
                sh_link,
                sh_info,
                sh_addralign,
                sh_entsize,
            })
        }
    }
}

/// Retrieves a null-terminated string from string table data.
pub fn get_string_from_table(table_data: &[u8], offset: u32) -> Result<&str, KernelError> {
    let start = offset as usize;
    if start >= table_data.len() {
        return Err(KernelError::Fault);
    }

    let mut end = start;
    while end < table_data.len() && table_data[end] != 0 {
        end += 1;
    }

    core::str::from_utf8(&table_data[start..end]).map_err(|_| KernelError::InvalidArgument)
}

/// Locates a section header by ASCII name string.
pub fn find_section_by_name(
    data: &[u8],
    hdr: &ElfHeaderInfo,
    name: &str,
) -> Result<ElfSectionHeader, KernelError> {
    if hdr.shstrndx >= hdr.shnum {
        return Err(KernelError::NotFound);
    }

    let shstr_sec = parse_section_header(data, hdr, hdr.shstrndx)?;
    let str_offset = shstr_sec.sh_offset as usize;
    let str_size = shstr_sec.sh_size as usize;

    if str_offset + str_size > data.len() {
        return Err(KernelError::Fault);
    }
    let shstr_bytes = &data[str_offset..str_offset + str_size];

    for idx in 0..hdr.shnum {
        let sec = parse_section_header(data, hdr, idx)?;
        if let Ok(sec_name) = get_string_from_table(shstr_bytes, sec.name_offset) {
            if sec_name == name {
                return Ok(sec);
            }
        }
    }

    Err(KernelError::NotFound)
}

/// Parses a symbol table entry by index from a `.symtab` section.
pub fn parse_symbol(
    data: &[u8],
    hdr: &ElfHeaderInfo,
    symtab_sec: &ElfSectionHeader,
    sym_idx: usize,
) -> Result<ElfSymbolInfo, KernelError> {
    let entsize = if symtab_sec.sh_entsize > 0 {
        symtab_sec.sh_entsize as usize
    } else {
        match hdr.class {
            ElfClass::Elf64 => 24,
            ElfClass::Elf32 => 16,
        }
    };

    let offset = (symtab_sec.sh_offset as usize) + (sym_idx * entsize);
    if offset + entsize > data.len() {
        return Err(KernelError::Fault);
    }

    match hdr.class {
        ElfClass::Elf64 => {
            let name_offset = read_u32_le(data, offset)?;
            let info = data[offset + 4];
            let _other = data[offset + 5];
            let shndx = read_u16_le(data, offset + 6)?;
            let value = read_u64_le(data, offset + 8)?;
            let size = read_u64_le(data, offset + 16)?;

            Ok(ElfSymbolInfo {
                name_offset,
                value,
                size,
                shndx,
                sym_type: info & 0x0F,
                sym_bind: info >> 4,
            })
        }
        ElfClass::Elf32 => {
            let name_offset = read_u32_le(data, offset)?;
            let value = read_u32_le(data, offset + 4)? as u64;
            let size = read_u32_le(data, offset + 8)? as u64;
            let info = data[offset + 12];
            let _other = data[offset + 13];
            let shndx = read_u16_le(data, offset + 14)?;

            Ok(ElfSymbolInfo {
                name_offset,
                value,
                size,
                shndx,
                sym_type: info & 0x0F,
                sym_bind: info >> 4,
            })
        }
    }
}

/// Parses a relocation entry from a `.rela` or `.rel` section.
pub fn parse_relocation(
    data: &[u8],
    hdr: &ElfHeaderInfo,
    rel_sec: &ElfSectionHeader,
    rel_idx: usize,
) -> Result<ElfRelocInfo, KernelError> {
    let is_rela = rel_sec.sh_type == SHT_RELA;
    let entsize = if rel_sec.sh_entsize > 0 {
        rel_sec.sh_entsize as usize
    } else {
        match (hdr.class, is_rela) {
            (ElfClass::Elf64, true) => 24,
            (ElfClass::Elf64, false) => 16,
            (ElfClass::Elf32, true) => 12,
            (ElfClass::Elf32, false) => 8,
        }
    };

    let offset = (rel_sec.sh_offset as usize) + (rel_idx * entsize);
    if offset + entsize > data.len() {
        return Err(KernelError::Fault);
    }

    match (hdr.class, is_rela) {
        (ElfClass::Elf64, true) => {
            let r_offset = read_u64_le(data, offset)?;
            let r_info = read_u64_le(data, offset + 8)?;
            let r_addend = read_u64_le(data, offset + 16)? as i64;
            let sym_idx = (r_info >> 32) as usize;
            let rel_type = (r_info & 0xFFFF_FFFF) as u32;

            Ok(ElfRelocInfo {
                offset: r_offset,
                sym_idx,
                rel_type,
                addend: r_addend,
            })
        }
        (ElfClass::Elf64, false) => {
            let r_offset = read_u64_le(data, offset)?;
            let r_info = read_u64_le(data, offset + 8)?;
            let sym_idx = (r_info >> 32) as usize;
            let rel_type = (r_info & 0xFFFF_FFFF) as u32;

            Ok(ElfRelocInfo {
                offset: r_offset,
                sym_idx,
                rel_type,
                addend: 0,
            })
        }
        (ElfClass::Elf32, true) => {
            let r_offset = read_u32_le(data, offset)? as u64;
            let r_info = read_u32_le(data, offset + 4)?;
            let r_addend = read_u32_le(data, offset + 8)? as i64;
            let sym_idx = (r_info >> 8) as usize;
            let rel_type = (r_info & 0xFF) as u32;

            Ok(ElfRelocInfo {
                offset: r_offset,
                sym_idx,
                rel_type,
                addend: r_addend,
            })
        }
        (ElfClass::Elf32, false) => {
            let r_offset = read_u32_le(data, offset)? as u64;
            let r_info = read_u32_le(data, offset + 4)?;
            let sym_idx = (r_info >> 8) as usize;
            let rel_type = (r_info & 0xFF) as u32;

            Ok(ElfRelocInfo {
                offset: r_offset,
                sym_idx,
                rel_type,
                addend: 0,
            })
        }
    }
}
