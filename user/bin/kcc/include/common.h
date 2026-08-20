/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KCC_COMMON_H
#define _KCC_COMMON_H

#include <stddef.h>
#include <stdint.h>

#define MAX_CODE_SIZE 16384
#define MAX_DATA_SIZE 8192
#define MAX_SOURCE_SIZE 32768

extern char src_buf[MAX_SOURCE_SIZE];
extern unsigned char code_buf[MAX_CODE_SIZE];
extern unsigned char data_buf[MAX_DATA_SIZE];
extern int code_idx;
extern int data_idx;

int k_strcmp(const char *s1, const char *s2);
int k_strlen(const char *s);
void k_strcpy(char *dest, const char *src);
void k_memcpy(char *dest, const char *src, int n);
void k_memset(char *dest, int val, int n);

void print_str(const char *s);
void print_num(int val);

void write_u8(char *buf, int offset, int val);
void write_u16(char *buf, int offset, int val);
void write_u32(char *buf, int offset, int val);
void write_u64(char *buf, int offset, unsigned long val);

#endif /* _KCC_COMMON_H */
