/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "elf.h"

#include "common.h"
#include "symbols.h"

#include <syscall.h>

int write_elf_executable(const char *output_path) {
    int main_offset = lookup_function("main");
    if (main_offset == 0 - 1) {
        print_str("Error: main function not defined\n");
        return -1;
    }

    unsigned long final_code_size = (unsigned long)code_idx;
    unsigned long base_vaddr = 0x40000000;
    unsigned long headers_size = 120;
    unsigned long text_start_vaddr = base_vaddr + headers_size;
    unsigned long data_start_vaddr = text_start_vaddr + final_code_size;

    /* Patch global variables virtual addresses in code segment */
    int i = 0;
    while (i < val_patch_count) {
        int patch_address = val_patch_addresses[i];
        int offset = val_patch_offsets[i];
        unsigned long target_vaddr = data_start_vaddr + offset;
        k_memcpy((char *)(code_buf + patch_address), (char *)&target_vaddr, 8);
        i = i + 1;
    }

    char header_buf[120];
    k_memset(header_buf, 0, 120);

    /* e_ident: \x7FELF, ELFCLASS64, ELFDATA2LSB, EV_CURRENT */
    write_u8(header_buf, 0, 127);
    write_u8(header_buf, 1, 69);
    write_u8(header_buf, 2, 76);
    write_u8(header_buf, 3, 70);
    write_u8(header_buf, 4, 2);
    write_u8(header_buf, 5, 1);
    write_u8(header_buf, 6, 1);

    /* e_type (ET_EXEC = 2), e_machine (EM_X86_64 = 0x3E), e_version (1) */
    write_u16(header_buf, 16, 2);
    write_u16(header_buf, 18, 0x3E);
    write_u32(header_buf, 20, 1);

    /* e_entry (entry point address) */
    write_u64(header_buf, 24, text_start_vaddr + main_offset);
    /* e_phoff (program header offset: 64) */
    write_u64(header_buf, 32, 64);

    /* e_ehsize (64), e_phentsize (56), e_phnum (1) */
    write_u16(header_buf, 52, 64);
    write_u16(header_buf, 54, 56);
    write_u16(header_buf, 56, 1);

    /* Phdr: p_type (PT_LOAD = 1), p_flags (PF_R | PF_W | PF_X = 7) */
    write_u32(header_buf, 64, 1);
    write_u32(header_buf, 68, 7);

    /* Phdr: p_offset (0), p_vaddr, p_paddr */
    write_u64(header_buf, 72, 0);
    write_u64(header_buf, 80, base_vaddr);
    write_u64(header_buf, 88, base_vaddr);

    /* Phdr: p_filesz, p_memsz, p_align (4096) */
    unsigned long total_filesz = 120 + final_code_size + data_idx;
    write_u64(header_buf, 96, total_filesz);
    write_u64(header_buf, 104, total_filesz);
    write_u64(header_buf, 112, 4096);

    int out_fd = sys_open(output_path, 1, 0755);
    if (out_fd < 0) {
        print_str("Error: Could not open output file ");
        print_str(output_path);
        print_str("\n");
        return -1;
    }

    sys_write(out_fd, header_buf, 120);
    sys_write(out_fd, (char *)code_buf, final_code_size);
    sys_write(out_fd, (char *)data_buf, data_idx);
    sys_close(out_fd);

    return 0;
}
