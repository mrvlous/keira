// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
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
