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

void skip_whitespace(void) {
    while (*src_ptr != '\0') {
        if (*src_ptr == ' ' || *src_ptr == '\t' || *src_ptr == '\r') {
            src_ptr++;
        } else if (*src_ptr == '\n') {
            line_num++;
            src_ptr++;
        } else if (*src_ptr == '#') {
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr++;
            }
            if (*src_ptr == '\n') {
                line_num++;
                src_ptr++;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '/') {
            src_ptr += 2;
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr++;
            }
            if (*src_ptr == '\n') {
                line_num++;
                src_ptr++;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '*') {
            src_ptr += 2;
            while (*src_ptr && !(*src_ptr == '*' && *(src_ptr + 1) == '/')) {
                if (*src_ptr == '\n')
                    line_num++;
                src_ptr++;
            }
            if (*src_ptr) {
                src_ptr += 2;
            }
        } else {
            break;
        }
    }
}
