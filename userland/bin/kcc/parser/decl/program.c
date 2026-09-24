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

int current_is_main = 0;

/* Global Declarations: Variables, Arrays, and Function Definitions */
void compile_global_declarations(void) {
    tok = next_token();
    while (tok != TOK_EOF) {
        while (tok == TOK_CONST || tok == TOK_EXTERN) {
            match(tok);
        }

        if (tok == TOK_SEMICOLON) {
            match(TOK_SEMICOLON);
            continue;
        }

        if (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_VOID || tok == TOK_SHORT ||
            tok == TOK_LONG || tok == TOK_UNSIGNED || tok == TOK_SIGNED) {
            int type_tok = tok;
            match(type_tok);
            while (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_SHORT || tok == TOK_LONG ||
                   tok == TOK_UNSIGNED || tok == TOK_SIGNED) {
                match(tok);
            }

            int is_ptr = 0;
            while (tok == TOK_STAR || tok == TOK_CONST) {
                if (tok == TOK_STAR) {
                    is_ptr = 1;
                }
                match(tok);
            }
            (void)is_ptr;

            char name[256];
            k_strcpy(name, token_string);
            match(TOK_IDENT);

            if (tok == TOK_LPAREN) {
                /* Function Declaration or Definition */
                match(TOK_LPAREN);
                clear_locals();

                int param_count = 0;
                if (tok != TOK_RPAREN) {
                    while (tok == TOK_CONST) {
                        match(TOK_CONST);
                    }
                    if (tok == TOK_VOID && *(src_ptr) == ')') {
                        match(TOK_VOID);
                    } else if (tok == TOK_ELLIPSIS) {
                        match(TOK_ELLIPSIS);
                    } else {
                        int p_type = tok;
                        match(p_type);
                        while (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_SHORT ||
                               tok == TOK_LONG || tok == TOK_UNSIGNED || tok == TOK_SIGNED) {
                            match(tok);
                        }
                        int p_is_ptr = 0;
                        while (tok == TOK_STAR || tok == TOK_CONST) {
                            if (tok == TOK_STAR) {
                                p_is_ptr = 1;
                            }
                            match(tok);
                        }
                        (void)p_is_ptr;
                        char p_name[256];
                        if (tok == TOK_IDENT) {
                            k_strcpy(p_name, token_string);
                            match(TOK_IDENT);
                            add_local(p_name, 8);
                        }
                        param_count++;

                        while (tok == TOK_COMMA) {
                            match(TOK_COMMA);
                            while (tok == TOK_CONST) {
                                match(TOK_CONST);
                            }
                            if (tok == TOK_ELLIPSIS) {
                                match(TOK_ELLIPSIS);
                                break;
                            }
                            int next_p_type = tok;
                            match(next_p_type);
                            while (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_SHORT ||
                                   tok == TOK_LONG || tok == TOK_UNSIGNED || tok == TOK_SIGNED) {
                                match(tok);
                            }
                            int next_p_is_ptr = 0;
                            while (tok == TOK_STAR || tok == TOK_CONST) {
                                if (tok == TOK_STAR) {
                                    next_p_is_ptr = 1;
                                }
                                match(tok);
                            }
                            (void)next_p_is_ptr;
                            char next_p_name[256];
                            if (tok == TOK_IDENT) {
                                k_strcpy(next_p_name, token_string);
                                match(TOK_IDENT);
                                add_local(next_p_name, 8);
                            }
                            param_count++;
                        }
                    }
                }
                match(TOK_RPAREN);

                if (tok == TOK_SEMICOLON) {
                    /* Function Prototype / Forward Declaration */
                    match(TOK_SEMICOLON);
                    clear_locals();
                    continue;
                }

                /* Actual Function Definition */
                int is_main = (k_strcmp(name, "main") == 0);
                current_is_main = is_main;

                add_function(name, code_idx);
                emit_func_prologue();

                /* Save incoming parameter registers to local stack slots */
                int p;
                for (p = 0; p < param_count && p < 6; p++) {
                    int slot_offset = local_offsets[p];
                    emit_param_save(p, slot_offset);
                }

                match(TOK_LBRACE);
                block();
                match(TOK_RBRACE);

                emit_func_epilogue(is_main);
                current_is_main = 0;
            } else if (tok == TOK_LBRACKET) {
                /* Global Array: char buf[1024]; */
                match(TOK_LBRACKET);
                int size = (int)token_num;
                match(TOK_NUM);
                match(TOK_RBRACKET);
                add_global(name, size);
                match(TOK_SEMICOLON);
            } else if (tok == TOK_ASSIGN) {
                /* Initialized Global Variable: int x = 42; */
                match(TOK_ASSIGN);
                int offset = add_global(name, 8);
                long init_val = 0;
                int sign = 1;
                if (tok == TOK_MINUS) {
                    match(TOK_MINUS);
                    sign = -1;
                } else if (tok == TOK_PLUS) {
                    match(TOK_PLUS);
                }
                if (tok == TOK_NUM) {
                    init_val = sign * token_num;
                    match(TOK_NUM);
                }
                if (offset >= 0 && offset + 8 <= MAX_DATA_SIZE) {
                    write_u64((char *)(data_buf + offset), 0, (uint64_t)init_val);
                }
                match(TOK_SEMICOLON);
            } else {
                /* Uninitialized Global Variable: int x; */
                add_global(name, 8);
                match(TOK_SEMICOLON);
            }
        } else {
            tok = next_token();
        }
    }
}
