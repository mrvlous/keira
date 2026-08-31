/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void print_num(int val) {
    if (val < 0) {
        print_char(45);
        val = -val;
    }
    if (val / 10 != 0) {
        print_num(val / 10);
    }
    print_char(48 + (val % 10));
}

void print_str(char *s) {
    while (*s != 0) {
        print_char(*s);
        s = s + 1;
    }
}

int str_contains(char *str, char *pat) {
    int i = 0;
    while (*(str + i) != 0) {
        int j = 0;
        while (*(pat + j) != 0 && *(str + i + j) == *(pat + j)) {
            j = j + 1;
        }
        if (*(pat + j) == 0)
            return 1;
        i = i + 1;
    }
    return 0;
}

void check_and_print(int line_no, char *line, char *pat) {
    if (str_contains(line, pat)) {
        printf("[Match ");
        print_num(line_no);
        printf("] ");
        print_str(line);
        printf("\n");
    }
}

void main(void) {
    printf("Keira Text Search Pattern Matcher (grep)\n\n");
    printf("Pattern Query: \"Rust\"\n\n");

    char *pat = "Rust";
    check_and_print(1, "Keira Kernel Operating System", pat);
    check_and_print(2, "Pure Rust bare-metal hyper-architecture", pat);
    check_and_print(3, "Freestanding C Userland Toolchain (KCC)", pat);
    check_and_print(4, "Rust and C multi-language system runtime", pat);

    printf("\n[OK] Pattern search finished.\n");
}
