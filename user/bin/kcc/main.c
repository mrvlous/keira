/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
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

void _start(int argc, char **argv) {
    print_str("KCC (Keira C Compiler) Native Toolchain\n");

    /* Initialize compiler subsystems */
    code_idx = 0;
    data_idx = 0;
    init_symbols();

    const char *source_path = "/data/main.c";
    const char *output_path = "/apps/bin/app.elf";

    int arg_i = 1;
    while (arg_i < argc && argv && argv[arg_i]) {
        if (k_strcmp(argv[arg_i], "-o") == 0) {
            if (arg_i + 1 < argc && argv[arg_i + 1]) {
                output_path = argv[arg_i + 1];
                arg_i += 2;
                continue;
            }
        } else if (k_strcmp(argv[arg_i], "-v") == 0 || k_strcmp(argv[arg_i], "--version") == 0) {
            print_str("Keira C Compiler (KCC) Native v0.1.0\n");
            sys_exit(0);
        } else if (k_strcmp(argv[arg_i], "-h") == 0 || k_strcmp(argv[arg_i], "--help") == 0) {
            print_str("Usage: kcc [options] <source.c>\n");
            print_str("Options:\n");
            print_str("  -o <path>     Specify output ELF binary (default: /apps/bin/app.elf)\n");
            print_str("  -v, --version Display compiler version\n");
            print_str("  -h, --help    Display this help message\n");
            sys_exit(0);
        } else if (argv[arg_i][0] != '-') {
            source_path = argv[arg_i];
        }
        arg_i++;
    }

    int in_fd = sys_open(source_path, 0, 0);
    if (in_fd < 0 && argc < 2) {
        source_path = "/temp/main.c";
        in_fd = sys_open(source_path, 0, 0);
    }
    if (in_fd < 0) {
        print_str("Error: Could not open source file: ");
        print_str(source_path);
        print_str("\n");
        print_str("Usage: run /system/bin/kcc.elf <source.c> [-o output.elf]\n");
        sys_exit(1);
    }

    print_str("[INFO] Compiling source: ");
    print_str(source_path);
    print_str(" -> ");
    print_str(output_path);
    print_str("\n");

    k_memset(src_buf, 0, MAX_SOURCE_SIZE);
    int read_len = sys_read(in_fd, src_buf, MAX_SOURCE_SIZE - 1);
    sys_close(in_fd);
    if (read_len <= 0) {
        print_str("Error: Source file is empty\n");
        sys_exit(1);
    }

    init_lexer(src_buf);
    compile_global_declarations();

    /* Patch function calls relative offsets */
    int i = 0;
    while (i < patch_count) {
        int patch_address = patch_addresses[i];
        int address = lookup_function(patch_names + i * 32);
        if (address == -1) {
            print_str("Error: Undefined function reference: '");
            print_str(patch_names + i * 32);
            print_str("'\n");
            sys_exit(1);
        }
        int rel_offset = address - (patch_address + 4);
        k_memcpy((char *)(code_buf + patch_address), (char *)&rel_offset, 4);
        i++;
    }

    if (write_elf_executable(output_path) < 0) {
        sys_exit(1);
    }

    print_str("[DONE] Compilation Successful!\n");
    print_str("       Code size: ");
    print_num(code_idx);
    print_str(" bytes, Data size: ");
    print_num(data_idx);
    print_str(" bytes\n");
    print_str("       Functions compiled: ");
    print_num(function_count);
    print_str("\n");
    print_str("       Executable written to ");
    print_str(output_path);
    print_str("\n");
    sys_exit(0);
}
