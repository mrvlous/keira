/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "codegen.h"
#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "symbols.h"

#include <syscall.h>

/* Unary Expressions: !, ~, -, +, &, *, ++, -- */
void unary_expr(void) {
    if (tok == TOK_NOT) {
        match(TOK_NOT);
        unary_expr();
        emit_log_not();
    } else if (tok == TOK_TILDE) {
        match(TOK_TILDE);
        unary_expr();
        emit_bit_not();
    } else if (tok == TOK_MINUS) {
        match(TOK_MINUS);
        unary_expr();
        emit_neg();
    } else if (tok == TOK_PLUS) {
        match(TOK_PLUS);
        unary_expr();
    } else if (tok == TOK_STAR) {
        match(TOK_STAR);
        unary_expr();
        emit_deref(8);
    } else if (tok == TOK_AMP) {
        match(TOK_AMP);
        if (tok == TOK_IDENT) {
            char name[256];
            k_strcpy(name, token_string);
            match(TOK_IDENT);
            int loc = lookup_local(name);
            if (loc != 0) {
                emit_addr_local(loc);
            } else {
                int glob = lookup_global(name);
                if (glob == -1) {
                    error_msg("Undefined variable in address-of operator");
                    sys_exit(1);
                }
                emit_addr_global(glob);
            }
        } else {
            error_msg("Expected identifier after '&'");
            sys_exit(1);
        }
    } else if (tok == TOK_INC) {
        match(TOK_INC);
        unary_expr();
        emit_load_imm(1);
        emit_add();
    } else if (tok == TOK_DEC) {
        match(TOK_DEC);
        unary_expr();
        emit_load_imm(1);
        emit_sub();
    } else {
        postfix_expr();
    }
}
