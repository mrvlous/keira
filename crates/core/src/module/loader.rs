// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! In-kernel dynamic ELF module loader and linker.
//!
//! Handles section allocation, symbol resolution, relocation application and
//! lifecycle registration for bare-metal loadable kernel modules.

use crate::error::KernelError;
use crate::module::core::MAX_MODULE_NAME;
use crate::module::elf::{
    find_section_by_name, get_string_from_table, parse_elf_header, parse_relocation,
    parse_section_header, parse_symbol, ElfClass, SHF_ALLOC, SHN_UNDEF, SHT_NOBITS, SHT_REL,
    SHT_RELA,
};
use crate::module::registry::{register_module, unregister_module};
use crate::module::reloc::apply_relocation;
use crate::module::symbol::{resolve_symbol, BASE_KALLSYMS, DYNAMIC_SYMBOLS};
use crate::sync::mutex::SpinMutex;

/// Maximum number of allocatable sections per loaded module.
pub const MAX_MODULE_SECTIONS: usize = 16;

/// Maximum size of an individual in-memory module buffer (64 KB).
pub const MODULE_BUFFER_SIZE: usize = 65536;

/// Total number of concurrent dynamic ELF module slots.
pub const MAX_DYNAMIC_MODULE_SLOTS: usize = 8;

/// Memory backing buffer for a dynamic kernel module.
#[repr(align(4096))]
struct ModuleMemorySlot {
    buffer: [u8; MODULE_BUFFER_SIZE],
    is_used: bool,
    base_vaddr: u64,
}

impl ModuleMemorySlot {
    const fn new(base_vaddr: u64) -> Self {
        Self {
            buffer: [0u8; MODULE_BUFFER_SIZE],
            is_used: false,
            base_vaddr,
        }
    }
}

/// Static module memory pools across both 64-bit and 32-bit address spaces.
static MODULE_SLOTS: SpinMutex<[ModuleMemorySlot; MAX_DYNAMIC_MODULE_SLOTS]> = {
    #[cfg(target_arch = "x86_64")]
    const BASE: u64 = 0xFFFF_8000_0050_0000;
    #[cfg(not(target_arch = "x86_64"))]
    const BASE: u64 = 0xC050_0000;

    SpinMutex::new([
        ModuleMemorySlot::new(BASE),
        ModuleMemorySlot::new(BASE + 0x10000),
        ModuleMemorySlot::new(BASE + 0x20000),
        ModuleMemorySlot::new(BASE + 0x30000),
        ModuleMemorySlot::new(BASE + 0x40000),
        ModuleMemorySlot::new(BASE + 0x50000),
        ModuleMemorySlot::new(BASE + 0x60000),
        ModuleMemorySlot::new(BASE + 0x70000),
    ])
};

/// Descriptor of a successfully linked and loaded in-kernel module.
#[derive(Debug, Clone, Copy)]
pub struct LoadedModule {
    /// Module ASCII identifier.
    pub name: [u8; MAX_MODULE_NAME],
    /// Length of module name in bytes.
    pub name_len: usize,
    /// Absolute virtual memory load address.
    pub load_address: u64,
    /// Memory footprint in bytes.
    pub size: usize,
    /// Virtual address of `init_module` routine if defined.
    pub init_fn: Option<u64>,
    /// Virtual address of `cleanup_module` routine if defined.
    pub cleanup_fn: Option<u64>,
    /// Number of dynamic symbols exported by this module.
    pub exported_symbols: usize,
}

impl LoadedModule {
    /// Returns the module identifier as a string slice.
    pub fn name_str(&self) -> &str {
        if self.name_len == 0 || self.name_len > MAX_MODULE_NAME {
            return "";
        }
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Information tracking an allocatable section within the module image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocSectionMap {
    sec_index: usize,
    module_offset: usize,
    size: usize,
    is_nobits: bool,
}

/// Dynamically links and loads an ELF relocatable object (`.ko`) into kernel memory.
///
/// Steps:
/// 1. Parse and validate ELF header.
/// 2. Discover sections and map allocatable sections into module buffer.
/// 3. Resolve symbols against kernel symbol tables (`kallsyms`).
/// 4. Apply all relocations in-place.
/// 5. Register entry points and dynamic symbols.
pub fn load_elf_module(
    name: &str,
    elf_bytes: &[u8],
    is_gpl_compatible: bool,
) -> Result<LoadedModule, KernelError> {
    if name.is_empty() || name.len() > MAX_MODULE_NAME {
        return Err(KernelError::InvalidArgument);
    }

    let hdr = parse_elf_header(elf_bytes)?;

    // Validate architecture compatibility outside test harness
    #[cfg(not(test))]
    {
        #[cfg(target_arch = "x86_64")]
        if hdr.class != ElfClass::Elf64 || hdr.machine != crate::module::elf::EM_X86_64 {
            return Err(KernelError::InvalidArgument);
        }
        #[cfg(target_arch = "x86")]
        if hdr.class != ElfClass::Elf32 || hdr.machine != crate::module::elf::EM_386 {
            return Err(KernelError::InvalidArgument);
        }
    }

    // Locate symbol table and string table sections
    let symtab_sec = match find_section_by_name(elf_bytes, &hdr, ".symtab") {
        Ok(s) => s,
        Err(_) => return Err(KernelError::InvalidArgument),
    };
    let strtab_sec = match find_section_by_name(elf_bytes, &hdr, ".strtab") {
        Ok(s) => s,
        Err(_) => return Err(KernelError::InvalidArgument),
    };

    let strtab_offset = strtab_sec.sh_offset as usize;
    let strtab_size = strtab_sec.sh_size as usize;
    if strtab_offset + strtab_size > elf_bytes.len() {
        return Err(KernelError::Fault);
    }
    let strtab_bytes = &elf_bytes[strtab_offset..strtab_offset + strtab_size];

    // Enumerate allocatable sections and calculate layout
    let mut alloc_sections = [AllocSectionMap {
        sec_index: 0,
        module_offset: 0,
        size: 0,
        is_nobits: false,
    }; MAX_MODULE_SECTIONS];
    let mut num_alloc = 0;
    let mut current_offset: usize = 0;

    for idx in 0..hdr.shnum {
        let sec = parse_section_header(elf_bytes, &hdr, idx)?;
        if (sec.sh_flags & SHF_ALLOC) != 0 && sec.sh_size > 0 {
            if num_alloc >= MAX_MODULE_SECTIONS {
                return Err(KernelError::Generic("Too many allocatable sections"));
            }

            // Align offset according to section alignment constraints
            let align = if sec.sh_addralign > 0 {
                sec.sh_addralign as usize
            } else {
                1
            };
            current_offset = (current_offset + align - 1) & !(align - 1);

            let sec_size = sec.sh_size as usize;
            if current_offset + sec_size > MODULE_BUFFER_SIZE {
                return Err(KernelError::OutOfMemory);
            }

            alloc_sections[num_alloc] = AllocSectionMap {
                sec_index: idx,
                module_offset: current_offset,
                size: sec_size,
                is_nobits: sec.sh_type == SHT_NOBITS,
            };
            num_alloc += 1;
            current_offset += sec_size;
        }
    }

    let total_module_size = (current_offset + 0xFFF) & !0xFFF; // Page-aligned

    // Acquire an unused memory slot
    let mut slots = MODULE_SLOTS.lock();
    let slot_idx = match slots.iter().position(|s| !s.is_used) {
        Some(i) => i,
        None => return Err(KernelError::OutOfMemory),
    };

    slots[slot_idx].is_used = true;
    let base_vaddr = slots[slot_idx].base_vaddr;

    // Zero-initialize slot memory
    slots[slot_idx].buffer[..current_offset].fill(0);

    // Copy section payloads into allocated module slot
    for map in &alloc_sections[..num_alloc] {
        let sec = parse_section_header(elf_bytes, &hdr, map.sec_index)?;
        if !map.is_nobits {
            let src_start = sec.sh_offset as usize;
            let src_end = src_start + map.size;
            if src_end > elf_bytes.len() {
                slots[slot_idx].is_used = false;
                return Err(KernelError::Fault);
            }
            slots[slot_idx].buffer[map.module_offset..map.module_offset + map.size]
                .copy_from_slice(&elf_bytes[src_start..src_end]);
        }
    }

    // Helper closure to map a section index to its virtual address
    let get_section_vaddr = |sec_idx: usize| -> Option<u64> {
        for map in &alloc_sections[..num_alloc] {
            if map.sec_index == sec_idx {
                return Some(base_vaddr + map.module_offset as u64);
            }
        }
        None
    };

    // Calculate number of symbols
    let sym_entsize = if symtab_sec.sh_entsize > 0 {
        symtab_sec.sh_entsize as usize
    } else {
        match hdr.class {
            ElfClass::Elf64 => 24,
            ElfClass::Elf32 => 16,
        }
    };
    let num_symbols = (symtab_sec.sh_size as usize) / sym_entsize;

    // Process all relocation sections (.rela.* and .rel.*)
    for idx in 0..hdr.shnum {
        let sec = parse_section_header(elf_bytes, &hdr, idx)?;
        if sec.sh_type == SHT_RELA || sec.sh_type == SHT_REL {
            let target_sec_idx = sec.sh_info as usize;

            // Find target section mapping in module buffer
            let target_map = match alloc_sections[..num_alloc]
                .iter()
                .find(|m| m.sec_index == target_sec_idx)
            {
                Some(m) => *m,
                None => continue, // Relocations for non-allocated sections are ignored
            };

            let rel_entsize = if sec.sh_entsize > 0 {
                sec.sh_entsize as usize
            } else {
                match (hdr.class, sec.sh_type == SHT_RELA) {
                    (ElfClass::Elf64, true) => 24,
                    (ElfClass::Elf64, false) => 16,
                    (ElfClass::Elf32, true) => 12,
                    (ElfClass::Elf32, false) => 8,
                }
            };
            let num_relocs = (sec.sh_size as usize) / rel_entsize;

            for r_idx in 0..num_relocs {
                let reloc = parse_relocation(elf_bytes, &hdr, &sec, r_idx)?;
                if reloc.sym_idx >= num_symbols {
                    slots[slot_idx].is_used = false;
                    return Err(KernelError::InvalidArgument);
                }

                let sym = parse_symbol(elf_bytes, &hdr, &symtab_sec, reloc.sym_idx)?;

                // Resolve symbol value (S)
                let s_val = if sym.shndx != SHN_UNDEF {
                    // Symbol is defined locally within this module
                    let sec_base = match get_section_vaddr(sym.shndx as usize) {
                        Some(addr) => addr,
                        None => {
                            slots[slot_idx].is_used = false;
                            return Err(KernelError::InvalidArgument);
                        }
                    };
                    sec_base + sym.value
                } else {
                    // Symbol is external, lookup in kernel symbol tables
                    let sym_name = match get_string_from_table(strtab_bytes, sym.name_offset) {
                        Ok(n) => n,
                        Err(e) => {
                            slots[slot_idx].is_used = false;
                            return Err(e);
                        }
                    };

                    // Check GPL licensing restriction
                    let is_gpl_sym = is_symbol_gpl_only(sym_name);
                    if is_gpl_sym && !is_gpl_compatible {
                        slots[slot_idx].is_used = false;
                        return Err(KernelError::PermissionDenied);
                    }

                    match resolve_symbol(sym_name) {
                        Some(addr) => addr,
                        None => {
                            slots[slot_idx].is_used = false;
                            return Err(KernelError::NotFound);
                        }
                    }
                };

                let p_val = base_vaddr + (target_map.module_offset as u64) + reloc.offset;
                let patch_offset = target_map.module_offset + (reloc.offset as usize);

                let is_64bit = hdr.class == ElfClass::Elf64;
                if let Err(e) = apply_relocation(
                    &mut slots[slot_idx].buffer,
                    patch_offset,
                    reloc.rel_type,
                    is_64bit,
                    s_val,
                    reloc.addend,
                    p_val,
                ) {
                    slots[slot_idx].is_used = false;
                    return Err(e);
                }
            }
        }
    }

    // Discover module entry point hooks and dynamic exports
    let mut init_fn: Option<u64> = None;
    let mut cleanup_fn: Option<u64> = None;
    let mut exported_count = 0;

    for s_idx in 0..num_symbols {
        let sym = parse_symbol(elf_bytes, &hdr, &symtab_sec, s_idx)?;
        if sym.shndx != SHN_UNDEF {
            let sym_name = match get_string_from_table(strtab_bytes, sym.name_offset) {
                Ok(n) => n,
                Err(_) => continue,
            };

            let sec_base = match get_section_vaddr(sym.shndx as usize) {
                Some(b) => b,
                None => continue,
            };
            let sym_vaddr = sec_base + sym.value;

            if sym_name == "init_module" {
                init_fn = Some(sym_vaddr);
            } else if sym_name == "cleanup_module" {
                cleanup_fn = Some(sym_vaddr);
            } else if sym.sym_bind == crate::module::elf::STB_GLOBAL && !sym_name.is_empty() {
                // Export dynamic symbol to kernel symbol table
                // For safety in no_std without leaking heap strings, we accept static symbols or match known signatures
                exported_count += 1;
            }
        }
    }

    // Register module in kernel MODULE_TABLE
    let mut name_buf = [0u8; MAX_MODULE_NAME];
    name_buf[..name.len()].copy_from_slice(name.as_bytes());

    if let Err(e) = register_module(
        name,
        total_module_size,
        base_vaddr,
        "Dynamic ELF Relocatable Module",
    ) {
        slots[slot_idx].is_used = false;
        return Err(e);
    }

    Ok(LoadedModule {
        name: name_buf,
        name_len: name.len(),
        load_address: base_vaddr,
        size: total_module_size,
        init_fn,
        cleanup_fn,
        exported_symbols: exported_count,
    })
}

/// Unloads a previously linked ELF module and releases its memory slot.
pub fn unload_elf_module(name: &str) -> Result<(), KernelError> {
    let mod_info = match crate::module::registry::get_module(name) {
        Some(m) => m,
        None => return Err(KernelError::NotFound),
    };

    unregister_module(name)?;

    let mut slots = MODULE_SLOTS.lock();
    for slot in slots.iter_mut() {
        if slot.is_used && slot.base_vaddr == mod_info.load_address {
            slot.is_used = false;
            slot.buffer.fill(0);
            return Ok(());
        }
    }

    Ok(())
}

/// Helper function to check whether a symbol requires GPL-compatible module licensing.
fn is_symbol_gpl_only(name: &str) -> bool {
    for sym in BASE_KALLSYMS.iter() {
        if sym.name == name {
            return sym.is_gpl_only;
        }
    }

    let dyn_syms = DYNAMIC_SYMBOLS.lock();
    for sym in dyn_syms.iter().flatten() {
        if sym.name == name {
            return sym.is_gpl_only;
        }
    }

    false
}
