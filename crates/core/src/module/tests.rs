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

//! Unit test suite for loadable kernel module lifecycle and symbol resolution.

use super::core::state::ModuleState;
use super::registry::manager::{get_module, register_module, unregister_module};
use super::registry::table::MODULE_TABLE;
use super::symbol::resolve::{lookup_symbol_by_addr, register_symbol, resolve_symbol};
use crate::error::KernelError;

#[test]
fn test_module_lifecycle() {
    let name = "test_mod_virt";
    assert!(get_module(name).is_none());

    let _idx = register_module(name, 4096, 0x1000_0000, "Virtual Test Driver")
        .expect("Module registration should succeed");

    let found = get_module(name).expect("Module should be found in registry");
    assert_eq!(found.name_str(), name);
    assert_eq!(found.size, 4096);
    assert_eq!(found.state, ModuleState::Live);
    assert_eq!(found.ref_count, 0);

    // Verify duplicate registration error
    assert!(register_module(name, 4096, 0x2000_0000, "Duplicate").is_err());

    // Unregister module
    assert!(unregister_module(name).is_ok());
    assert!(get_module(name).is_none());
}

#[test]
fn test_symbol_resolution() {
    assert_eq!(resolve_symbol("vga_print_str"), Some(0xFFFF_8000_0010_1000));
    assert_eq!(
        lookup_symbol_by_addr(0xFFFF_8000_0010_1000),
        Some("vga_print_str")
    );

    let dyn_name = "custom_irq_handler";
    assert!(resolve_symbol(dyn_name).is_none());
    assert!(register_symbol(dyn_name, 0xFFFF_8000_0099_0000, false).is_ok());
    assert_eq!(resolve_symbol(dyn_name), Some(0xFFFF_8000_0099_0000));
    assert_eq!(lookup_symbol_by_addr(0xFFFF_8000_0099_0000), Some(dyn_name));
}

#[test]
fn test_module_busy_refcount() {
    let name = "busy_driver";
    let _ = register_module(name, 8192, 0x3000_0000, "Busy Driver");

    // Artificially raise ref_count in table
    {
        let mut table = MODULE_TABLE.lock();
        for m in table.iter_mut().flatten() {
            if m.matches_name(name) {
                m.ref_count = 3;
            }
        }
    }

    // Unload should fail with DeviceBusy
    let res = unregister_module(name);
    assert_eq!(res, Err(KernelError::DeviceBusy));

    // Restore ref_count to 0 and unload
    {
        let mut table = MODULE_TABLE.lock();
        for m in table.iter_mut().flatten() {
            if m.matches_name(name) {
                m.ref_count = 0;
            }
        }
    }
    assert!(unregister_module(name).is_ok());
}

#[test]
fn test_elf_relocations_computation() {
    use super::reloc::{
        apply_relocation, R_386_32, R_386_PC32, R_X86_64_32, R_X86_64_64, R_X86_64_PC32,
    };

    let mut buf = [0u8; 32];

    // Test R_X86_64_64: S + A
    let s = 0xFFFF_8000_0010_1000;
    let a = 0x20;
    let p = 0xFFFF_8000_0050_0000;
    assert!(apply_relocation(&mut buf, 0, R_X86_64_64, true, s, a, p).is_ok());
    let val64 = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    assert_eq!(val64, 0xFFFF_8000_0010_1020);

    // Test R_X86_64_PC32: S + A - P
    assert!(apply_relocation(&mut buf, 8, R_X86_64_PC32, true, s, a, p).is_ok());
    let disp32 = i32::from_le_bytes(buf[8..12].try_into().unwrap());
    let expected_disp = ((s as i64 + a) - p as i64) as i32;
    assert_eq!(disp32, expected_disp);

    // Test R_X86_64_32: S + A
    assert!(apply_relocation(&mut buf, 12, R_X86_64_32, true, 0x1000, 0x24, p).is_ok());
    let val32 = u32::from_le_bytes(buf[12..16].try_into().unwrap());
    assert_eq!(val32, 0x1024);

    // Test 32-bit R_386_32: S + A
    assert!(apply_relocation(
        &mut buf,
        16,
        R_386_32,
        false,
        0xC010_1000,
        0x10,
        0xC050_0000
    )
    .is_ok());
    let val_386 = u32::from_le_bytes(buf[16..20].try_into().unwrap());
    assert_eq!(val_386, 0xC010_1010);

    // Test 32-bit R_386_PC32: S + A - P
    assert!(apply_relocation(&mut buf, 20, R_386_PC32, false, 0xC010_2000, 0, 0xC010_1000).is_ok());
    let disp_386 = i32::from_le_bytes(buf[20..24].try_into().unwrap());
    assert_eq!(disp_386, 0x1000);
}

#[test]
fn test_elf_header_parsing_and_validation() {
    use super::elf::{parse_elf_header, ElfClass, ELF_MAGIC, EM_386, EM_X86_64, ET_REL};

    // Valid 64-bit relocatable header
    let mut hdr64 = [0u8; 64];
    hdr64[0..4].copy_from_slice(&ELF_MAGIC);
    hdr64[4] = 2; // Elf64
    hdr64[5] = 1; // Little-endian
    hdr64[6] = 1; // EV_CURRENT
    hdr64[16..18].copy_from_slice(&ET_REL.to_le_bytes());
    hdr64[18..20].copy_from_slice(&EM_X86_64.to_le_bytes());
    hdr64[40..48].copy_from_slice(&64u64.to_le_bytes()); // shoff
    hdr64[58..60].copy_from_slice(&64u16.to_le_bytes()); // shentsize
    hdr64[60..62].copy_from_slice(&4u16.to_le_bytes()); // shnum
    hdr64[62..64].copy_from_slice(&1u16.to_le_bytes()); // shstrndx

    let res64 = parse_elf_header(&hdr64).expect("64-bit header parse should succeed");
    assert_eq!(res64.class, ElfClass::Elf64);
    assert_eq!(res64.machine, EM_X86_64);
    assert_eq!(res64.shoff, 64);
    assert_eq!(res64.shnum, 4);

    // Valid 32-bit relocatable header
    let mut hdr32 = [0u8; 52];
    hdr32[0..4].copy_from_slice(&ELF_MAGIC);
    hdr32[4] = 1; // Elf32
    hdr32[5] = 1; // Little-endian
    hdr32[6] = 1; // EV_CURRENT
    hdr32[16..18].copy_from_slice(&ET_REL.to_le_bytes());
    hdr32[18..20].copy_from_slice(&EM_386.to_le_bytes());
    hdr32[32..36].copy_from_slice(&52u32.to_le_bytes()); // shoff
    hdr32[46..48].copy_from_slice(&40u16.to_le_bytes()); // shentsize
    hdr32[48..50].copy_from_slice(&3u16.to_le_bytes()); // shnum
    hdr32[50..52].copy_from_slice(&1u16.to_le_bytes()); // shstrndx

    let res32 = parse_elf_header(&hdr32).expect("32-bit header parse should succeed");
    assert_eq!(res32.class, ElfClass::Elf32);
    assert_eq!(res32.machine, EM_386);
    assert_eq!(res32.shoff, 52);
    assert_eq!(res32.shnum, 3);

    // Invalid magic should fail
    let mut bad_hdr = hdr64;
    bad_hdr[0] = 0;
    assert!(parse_elf_header(&bad_hdr).is_err());
}

#[test]
fn test_synthetic_elf_module_loading_and_unloading() {
    use super::elf::*;
    use super::loader::{load_elf_module, unload_elf_module};

    // Construct a synthetic 64-bit relocatable ELF module
    let mut elf = [0u8; 1024];

    // ELF Header (64 bytes)
    elf[0..4].copy_from_slice(&ELF_MAGIC);
    elf[4] = 2; // Elf64
    elf[5] = 1; // Little-endian
    elf[6] = 1; // EV_CURRENT
    elf[16..18].copy_from_slice(&ET_REL.to_le_bytes());
    elf[18..20].copy_from_slice(&EM_X86_64.to_le_bytes());
    elf[40..48].copy_from_slice(&64u64.to_le_bytes()); // shoff = 64
    elf[58..60].copy_from_slice(&64u16.to_le_bytes()); // shentsize = 64
    elf[60..62].copy_from_slice(&6u16.to_le_bytes()); // shnum = 6
    elf[62..64].copy_from_slice(&2u16.to_le_bytes()); // shstrndx = 2

    // Section 1: .text payload (offset 448, size 16)
    let text_off = 448usize;
    elf[text_off..text_off + 16]
        .copy_from_slice(&[0x48, 0xC7, 0xC0, 0, 0, 0, 0, 0xC3, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Section 2: .shstrtab (offset 480, size 48)
    let shstr_off = 480usize;
    let shstr_bytes = b"\0.text\0.shstrtab\0.symtab\0.strtab\0.rela.text\0";
    elf[shstr_off..shstr_off + shstr_bytes.len()].copy_from_slice(shstr_bytes);

    // Section 3: .symtab (offset 544, size 72 = 3 * 24 bytes)
    let symtab_off = 544usize;
    // Sym 0: NULL (all 0s)
    // Sym 1: init_module (name_off 1, info 0x12 (GLOBAL FUNC), shndx 1, value 0, size 8)
    elf[symtab_off + 24..symtab_off + 28].copy_from_slice(&1u32.to_le_bytes()); // name_off
    elf[symtab_off + 28] = 0x12; // STB_GLOBAL (1) << 4 | STT_FUNC (2)
    elf[symtab_off + 30..symtab_off + 32].copy_from_slice(&1u16.to_le_bytes()); // shndx = 1 (.text)
    elf[symtab_off + 32..symtab_off + 40].copy_from_slice(&0u64.to_le_bytes()); // value = 0
    elf[symtab_off + 40..symtab_off + 48].copy_from_slice(&8u64.to_le_bytes()); // size = 8
                                                                                // Sym 2: vga_print_str external (name_off 13, info 0x10 (GLOBAL NOTYPE), shndx 0 (UNDEF))
    elf[symtab_off + 48..symtab_off + 52].copy_from_slice(&13u32.to_le_bytes());
    elf[symtab_off + 52] = 0x10;
    elf[symtab_off + 54..symtab_off + 56].copy_from_slice(&SHN_UNDEF.to_le_bytes());

    // Section 4: .strtab (offset 624, size 32)
    let strtab_off = 624usize;
    let strtab_bytes = b"\0init_module\0vga_print_str\0";
    elf[strtab_off..strtab_off + strtab_bytes.len()].copy_from_slice(strtab_bytes);

    // Section 5: .rela.text (offset 672, size 24 = 1 reloc)
    let rela_off = 672usize;
    let r_info = (2u64 << 32) | 10u64; // sym_idx 2 (vga_print_str), rel_type 10 (R_X86_64_32)
    elf[rela_off..rela_off + 8].copy_from_slice(&3u64.to_le_bytes()); // r_offset = 3
    elf[rela_off + 8..rela_off + 16].copy_from_slice(&r_info.to_le_bytes());
    elf[rela_off + 16..rela_off + 24].copy_from_slice(&0i64.to_le_bytes()); // r_addend = 0

    // Section Headers at shoff = 64
    // Sec 0: NULL
    // Sec 1: .text (name_off 1, SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, off 448, size 16, align 16)
    let s1 = 64 + 64;
    elf[s1..s1 + 4].copy_from_slice(&1u32.to_le_bytes());
    elf[s1 + 4..s1 + 8].copy_from_slice(&SHT_PROGBITS.to_le_bytes());
    elf[s1 + 8..s1 + 16].copy_from_slice(&(SHF_ALLOC | SHF_EXECINSTR).to_le_bytes());
    elf[s1 + 24..s1 + 32].copy_from_slice(&(text_off as u64).to_le_bytes());
    elf[s1 + 32..s1 + 40].copy_from_slice(&16u64.to_le_bytes());
    elf[s1 + 48..s1 + 56].copy_from_slice(&16u64.to_le_bytes());

    // Sec 2: .shstrtab (name_off 7, SHT_STRTAB, 0, off 480, size shstr_bytes.len())
    let s2 = 64 + 128;
    elf[s2..s2 + 4].copy_from_slice(&7u32.to_le_bytes());
    elf[s2 + 4..s2 + 8].copy_from_slice(&SHT_STRTAB.to_le_bytes());
    elf[s2 + 24..s2 + 32].copy_from_slice(&(shstr_off as u64).to_le_bytes());
    elf[s2 + 32..s2 + 40].copy_from_slice(&(shstr_bytes.len() as u64).to_le_bytes());

    // Sec 3: .symtab (name_off 17, SHT_SYMTAB, 0, off 544, size 72, link 4 (.strtab), entsize 24)
    let s3 = 64 + 192;
    elf[s3..s3 + 4].copy_from_slice(&17u32.to_le_bytes());
    elf[s3 + 4..s3 + 8].copy_from_slice(&SHT_SYMTAB.to_le_bytes());
    elf[s3 + 24..s3 + 32].copy_from_slice(&(symtab_off as u64).to_le_bytes());
    elf[s3 + 32..s3 + 40].copy_from_slice(&72u64.to_le_bytes());
    elf[s3 + 40..s3 + 44].copy_from_slice(&4u32.to_le_bytes()); // link = 4 (.strtab)
    elf[s3 + 56..s3 + 64].copy_from_slice(&24u64.to_le_bytes());

    // Sec 4: .strtab (name_off 25, SHT_STRTAB, 0, off 624, size strtab_bytes.len())
    let s4 = 64 + 256;
    elf[s4..s4 + 4].copy_from_slice(&25u32.to_le_bytes());
    elf[s4 + 4..s4 + 8].copy_from_slice(&SHT_STRTAB.to_le_bytes());
    elf[s4 + 24..s4 + 32].copy_from_slice(&(strtab_off as u64).to_le_bytes());
    elf[s4 + 32..s4 + 40].copy_from_slice(&(strtab_bytes.len() as u64).to_le_bytes());

    // Sec 5: .rela.text (name_off 33, SHT_RELA, 0, off 672, size 24, link 3 (.symtab), info 1 (.text), entsize 24)
    let s5 = 64 + 320;
    elf[s5..s5 + 4].copy_from_slice(&33u32.to_le_bytes());
    elf[s5 + 4..s5 + 8].copy_from_slice(&SHT_RELA.to_le_bytes());
    elf[s5 + 24..s5 + 32].copy_from_slice(&(rela_off as u64).to_le_bytes());
    elf[s5 + 32..s5 + 40].copy_from_slice(&24u64.to_le_bytes());
    elf[s5 + 40..s5 + 44].copy_from_slice(&3u32.to_le_bytes()); // link = 3 (.symtab)
    elf[s5 + 44..s5 + 48].copy_from_slice(&1u32.to_le_bytes()); // info = 1 (.text)
    elf[s5 + 56..s5 + 64].copy_from_slice(&24u64.to_le_bytes());

    // Load synthetic module into kernel
    let mod_name = "test_synth_ko";
    let loaded = load_elf_module(mod_name, &elf[..720], true)
        .expect("Synthetic ELF module loading should succeed");

    assert_eq!(loaded.name_str(), mod_name);
    assert!(loaded.load_address >= 0xC000_0000);
    assert!(loaded.init_fn.is_some());

    // Verify module appears in registry
    assert!(super::registry::manager::get_module(mod_name).is_some());

    // Unload module cleanly
    assert!(unload_elf_module(mod_name).is_ok());
    assert!(super::registry::manager::get_module(mod_name).is_none());
}
