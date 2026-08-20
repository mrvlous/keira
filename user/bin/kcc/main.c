/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "common.h"
#include "elf.h"
#include "lexer.h"
#include "parser.h"
#include "symbols.h"

#include <syscall.h>

void _start(void) {
    print_str("KCC (Keira C Compiler) for Keira Kernel\n");
    print_str("Compiling source: /data/main.c -> /apps/bin/app.elf\n");

    /* Initialize compiler state */
    code_idx = 0;
    data_idx = 0;
    global_count = 0;
    local_count = 0;
    function_count = 0;
    patch_count = 0;
    val_patch_count = 0;

    int in_fd = sys_open("/data/main.c", 0, 0);
    if (in_fd < 0) {
        in_fd = sys_open("/temp/main.c", 0, 0);
    }
    if (in_fd < 0) {
        print_str("Error: Could not open source file (/data/main.c)\n");
        print_str("Usage: Place target C code in /data/main.c and run 'kcc'\n");
        sys_exit(1);
    }

    k_memset(src_buf, 0, MAX_SOURCE_SIZE);
    int read_len = sys_read(in_fd, src_buf, MAX_SOURCE_SIZE - 1);
    sys_close(in_fd);
    if (read_len <= 0) {
        print_str("Error: Read empty or failed for /data/main.c\n");
        sys_exit(1);
    }

    src_ptr = src_buf;
    compile_global_declarations();

    /* Patch function calls relative offsets */
    int i = 0;
    while (i < patch_count) {
        int patch_address = patch_addresses[i];
        int address = lookup_function(patch_names + i * 32);
        if (address == 0 - 1) {
            print_str("Error: Undefined function reference: ");
            print_str(patch_names + i * 32);
            print_str("\n");
            sys_exit(1);
        }
        int rel_offset = address - (patch_address + 4);
        k_memcpy((char *)(code_buf + patch_address), (char *)&rel_offset, 4);
        i = i + 1;
    }

    if (write_elf_executable("/apps/bin/app.elf") < 0) {
        sys_exit(1);
    }

    print_str("Compilation Success! Created executable /apps/bin/app.elf\n");
    sys_exit(0);
}
