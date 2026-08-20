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

#include <syscall.h>

char src_buf[MAX_SOURCE_SIZE];
unsigned char code_buf[MAX_CODE_SIZE];
unsigned char data_buf[MAX_DATA_SIZE];
int code_idx = 0;
int data_idx = 0;

int k_strcmp(const char *s1, const char *s2) {
    while (*s1 == *s2) {
        if (*s1 == 0)
            return 0;
        s1 = s1 + 1;
        s2 = s2 + 1;
    }
    return *s1 - *s2;
}

int k_strlen(const char *s) {
    int len = 0;
    while (*s != 0) {
        len = len + 1;
        s = s + 1;
    }
    return len;
}

void k_strcpy(char *dest, const char *src) {
    while (*src != 0) {
        *dest = *src;
        dest = dest + 1;
        src = src + 1;
    }
    *dest = 0;
}

void k_memcpy(char *dest, const char *src, int n) {
    int i = 0;
    while (i < n) {
        *dest = *src;
        dest = dest + 1;
        src = src + 1;
        i = i + 1;
    }
}

void k_memset(char *dest, int val, int n) {
    int i = 0;
    while (i < n) {
        *dest = val;
        dest = dest + 1;
        i = i + 1;
    }
}

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

void write_u8(char *buf, int offset, int val) {
    buf[offset] = val;
}

void write_u16(char *buf, int offset, int val) {
    buf[offset] = val & 255;
    buf[offset + 1] = (val >> 8) & 255;
}

void write_u32(char *buf, int offset, int val) {
    buf[offset] = val & 255;
    buf[offset + 1] = (val >> 8) & 255;
    buf[offset + 2] = (val >> 16) & 255;
    buf[offset + 3] = (val >> 24) & 255;
}

void write_u64(char *buf, int offset, unsigned long val) {
    buf[offset] = val & 255;
    buf[offset + 1] = (val >> 8) & 255;
    buf[offset + 2] = (val >> 16) & 255;
    buf[offset + 3] = (val >> 24) & 255;
    buf[offset + 4] = (val >> 32) & 255;
    buf[offset + 5] = (val >> 40) & 255;
    buf[offset + 6] = (val >> 48) & 255;
    buf[offset + 7] = (val >> 56) & 255;
}
