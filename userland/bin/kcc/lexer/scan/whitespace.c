// SPDX-License-Identifier: GPL-2.0-only

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
