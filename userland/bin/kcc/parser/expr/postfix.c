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

#include "codegen.h"
#include "common.h"
#include "lexer.h"
#include "parser.h"

/* Postfix Expressions: Array indexing, Post-Increment/Decrement */
void postfix_expr(void) {
    primary_expr();
    while (tok == TOK_LBRACKET || tok == TOK_INC || tok == TOK_DEC) {
        if (tok == TOK_LBRACKET) {
            /* Array indexing: arr[index] */
            match(TOK_LBRACKET);
            emit_push_rax(); /* base address or pointer */
            expression();    /* index */
            match(TOK_RBRACKET);

            /* index in rax, base in top-of-stack */
            emit_pop_rcx(); /* rcx = base */
            emit_add();     /* rax = base + index (dual-arch safe) */
            emit_deref(1);  /* byte deref */
        } else if (tok == TOK_INC) {
            match(TOK_INC);
            /* Post-increment: rax holds original value */
            emit_push_rax();
            emit_load_imm(1);
            emit_pop_rcx();
            emit_add();
        } else if (tok == TOK_DEC) {
            match(TOK_DEC);
            emit_push_rax();
            emit_load_imm(1);
            emit_pop_rcx();
            emit_sub();
        }
    }
}
