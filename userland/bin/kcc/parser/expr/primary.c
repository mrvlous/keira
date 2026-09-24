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

/* Primary Expressions: Literals, Variables, Function Calls, Parenthesized */
void primary_expr(void) {
    if (tok == TOK_NUM) {
        emit_load_imm(token_num);
        match(TOK_NUM);
    } else if (tok == TOK_STRING) {
        int str_len = k_strlen(token_string) + 1;
        if (data_idx + str_len >= MAX_DATA_SIZE) {
            error_msg("Data segment overflow for string literal");
            sys_exit(1);
        }
        int str_offset = data_idx;
        k_memcpy((char *)(data_buf + data_idx), token_string, str_len);
        data_idx += str_len;

        emit_addr_global(str_offset);
        match(TOK_STRING);
    } else if (tok == TOK_SIZEOF) {
        match(TOK_SIZEOF);
        match(TOK_LPAREN);
        int sz = 8;
        if (tok == TOK_CHAR) {
            sz = 1;
            match(TOK_CHAR);
        } else if (tok == TOK_SHORT) {
            sz = 2;
            match(TOK_SHORT);
        } else if (tok == TOK_INT || tok == TOK_LONG || tok == TOK_UNSIGNED || tok == TOK_VOID) {
            sz = 8;
            match(tok);
        } else if (tok == TOK_IDENT) {
            int loc = lookup_local(token_string);
            if (loc != 0) {
                sz = 8;
            }
            match(TOK_IDENT);
        }
        if (tok == TOK_STAR) {
            match(TOK_STAR);
            sz = 8;
        }
        match(TOK_RPAREN);
        emit_load_imm(sz);
    } else if (tok == TOK_SYSCALL) {
        match(TOK_SYSCALL);
        match(TOK_LPAREN);
        assignment_expr();
        emit_push_rax(); /* syscall nr */
        match(TOK_COMMA);
        assignment_expr();
        emit_push_rax(); /* arg1 */
        match(TOK_COMMA);
        assignment_expr();
        emit_push_rax(); /* arg2 */
        match(TOK_COMMA);
        assignment_expr();
        emit_push_rax(); /* arg3 */
        match(TOK_RPAREN);

        emit_syscall_trampoline();
        return;
    } else if (tok == TOK_IDENT) {
        char name[256];
        k_strcpy(name, token_string);
        match(TOK_IDENT);

        /* Function Call */
        if (tok == TOK_LPAREN) {
            match(TOK_LPAREN);

            /* Special primitive syscall dispatcher: syscall(nr, arg1, arg2, arg3) */
            if (k_strcmp(name, "syscall") == 0) {
                assignment_expr();
                emit_push_rax(); /* syscall nr */
                match(TOK_COMMA);
                assignment_expr();
                emit_push_rax(); /* arg1 */
                match(TOK_COMMA);
                assignment_expr();
                emit_push_rax(); /* arg2 */
                match(TOK_COMMA);
                assignment_expr();
                emit_push_rax(); /* arg3 */
                match(TOK_RPAREN);

                emit_syscall_trampoline();
                return;
            }

            /* Regular Function Call with up to 6 arguments */
            int arg_count = 0;
            if (tok != TOK_RPAREN) {
                assignment_expr();
                emit_push_rax();
                arg_count++;
                while (tok == TOK_COMMA) {
                    match(TOK_COMMA);
                    assignment_expr();
                    emit_push_rax();
                    arg_count++;
                }
            }
            match(TOK_RPAREN);

            emit_call(name, arg_count);
        } else {
            /* Variable load */
            int loc_offset = lookup_local(name);
            if (loc_offset != 0) {
                emit_load_local(loc_offset, 8);
            } else {
                int glob_offset = lookup_global(name);
                if (glob_offset == -1) {
                    print_str("Error on line ");
                    print_num(line_num);
                    print_str(": Undefined identifier '");
                    print_str(name);
                    print_str("'\n");
                    sys_exit(1);
                }
                emit_load_global(glob_offset, 8);
            }
        }
    } else if (tok == TOK_LPAREN) {
        match(TOK_LPAREN);
        expression();
        match(TOK_RPAREN);
    } else {
        print_str("Error on line ");
        print_num(line_num);
        print_str(": Invalid primary expression at ");
        print_str(token_name(tok));
        print_str("\n");
        sys_exit(1);
    }
}
