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

#include <syscall.h>

void print_str(const char *s) {
    if (!s)
        return;
    while (*s != '\0') {
        sys_print_char(*s);
        s = s + 1;
    }
}

void print_num(int val) {
    char buf[16];
    int idx = 15;
    buf[15] = 0;
    if (val == 0) {
        print_str("0");
        return;
    }
    int is_neg = 0;
    if (val < 0) {
        is_neg = 1;
        val = 0 - val;
    }
    while (val > 0) {
        idx = idx - 1;
        buf[idx] = 48 + (val % 10);
        val = val / 10;
    }
    if (is_neg) {
        idx = idx - 1;
        buf[idx] = '-';
    }
    print_str(buf + idx);
}

void print_hex(uint64_t val) {
    char buf[20];
    int idx = 19;
    buf[19] = 0;
    if (val == 0) {
        print_str("0x0");
        return;
    }
    while (val > 0) {
        idx = idx - 1;
        int d = (int)(val & 15);
        if (d < 10)
            buf[idx] = 48 + d;
        else
            buf[idx] = 'a' + (d - 10);
        val = val >> 4;
    }
    idx = idx - 1;
    buf[idx] = 'x';
    idx = idx - 1;
    buf[idx] = '0';
    print_str(buf + idx);
}

void error_msg(const char *msg) {
    print_str("Error: ");
    print_str(msg);
    print_str("\n");
}
