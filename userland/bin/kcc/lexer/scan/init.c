// SPDX-License-Identifier: GPL-2.0-only

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
