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
#include "preproc.h"
#include "symbols.h"

#include <stdio.h>
#include <syscall.h>

int main(int argc, char **argv) {
    const char *source_path = NULL;
    const char *output_path = NULL;

    int arg_i = 1;
    while (arg_i < argc && argv && argv[arg_i]) {
        if (k_strcmp(argv[arg_i], "-o") == 0) {
            if (arg_i + 1 < argc && argv[arg_i + 1]) {
                output_path = argv[arg_i + 1];
                arg_i += 2;
                continue;
            } else {
                print_str("kcc: error: missing filename after '-o'\n");
                return 1;
            }
        } else if (k_strcmp(argv[arg_i], "-v") == 0 || k_strcmp(argv[arg_i], "--version") == 0) {
            print_str("kcc (Keira C Compiler) 0.6.0\n");
            return 0;
        } else if (k_strcmp(argv[arg_i], "-h") == 0 || k_strcmp(argv[arg_i], "--help") == 0) {
            print_str("Usage: kcc [options] <source.c>\n\n");
            print_str("Description:\n");
            print_str("  Keira native freestanding C compiler toolchain.\n\n");
            print_str("Options:\n");
            print_str("  -o <path>      Specify output ELF binary (default: /bin/app.elf)\n");
            print_str("  -v, --version  Display compiler version\n");
            print_str("  -h, --help     Display this help reference and exit\n");
            return 0;
        } else if (argv[arg_i][0] != '-') {
            source_path = argv[arg_i];
        } else {
            print_str("kcc: unrecognized command-line option '");
            print_str(argv[arg_i]);
            print_str("'\n");
            return 1;
        }
        arg_i++;
    }

    if (!source_path) {
        print_str("kcc: fatal error: no input files\n");
        print_str("compilation terminated.\n");
        return 1;
    }

    if (!output_path) {
        output_path = "/bin/app.elf";
    }

    FILE *in_fp = fopen(source_path, "r");
    if (!in_fp) {
        print_str("kcc: error: ");
        print_str(source_path);
        print_str(": No such file or directory\n");
        print_str("compilation terminated.\n");
        return 1;
    }

    /* Initialize compiler subsystems */
    code_idx = 0;
    data_idx = 0;
    init_symbols();

    print_str("[INFO] Compiling source: ");
    print_str(source_path);
    print_str(" -> ");
    print_str(output_path);
    print_str("\n");

    k_memset(src_buf, 0, MAX_SOURCE_SIZE);
    size_t read_len = fread(src_buf, 1, MAX_SOURCE_SIZE - 1, in_fp);
    fclose(in_fp);
    if (read_len == 0) {
        print_str("Error: Source file is empty\n");
        sys_exit(1);
        return 1;
    }

    /* Determine source directory for relative includes */
    char cur_dir[128];
    k_memset(cur_dir, 0, sizeof(cur_dir));
    int path_len = k_strlen(source_path);
    int last_slash = -1;
    int si;
    for (si = 0; si < path_len; si++) {
        if (source_path[si] == '/') {
            last_slash = si;
        }
    }
    if (last_slash >= 0) {
        k_memcpy(cur_dir, source_path, last_slash);
        cur_dir[last_slash] = '\0';
    } else {
        cur_dir[0] = '.';
        cur_dir[1] = '\0';
    }

    init_preprocessor();
    k_memset(prep_buf, 0, MAX_SOURCE_SIZE);
    int prep_len = preprocess_source(src_buf, prep_buf, MAX_SOURCE_SIZE, cur_dir);
    if (prep_len < 0) {
        print_str("Error: Preprocessing failed\n");
        sys_exit(1);
    }

    init_lexer(prep_buf);
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
    return 0;
}
