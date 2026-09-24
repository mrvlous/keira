/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "lexer.h"

char *src_ptr = 0;
int line_num = 1;
char token_string[256];
long token_num = 0;
int tok = 0;

void init_lexer(char *src) {
    src_ptr = src;
    line_num = 1;
    token_num = 0;
    token_string[0] = '\0';
    tok = 0;
}
