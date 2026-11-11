// SPDX-License-Identifier: GPL-2.0-only

#include "lexer.h"
#include "parser.h"

/* Statements & Control Flow */
void block(void) {
    while (tok != TOK_RBRACE && tok != TOK_EOF) {
        statement();
    }
}
