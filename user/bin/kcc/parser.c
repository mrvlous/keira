/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "parser.h"

#include "common.h"
#include "lexer.h"
#include "symbols.h"

#include <syscall.h>

void match(int expected) {
    if (tok == expected) {
        tok = next_token();
    } else {
        print_str("Error: Expected token ");
        print_num(expected);
        print_str(", got ");
        print_num(tok);
        print_str("\n");
        sys_exit(1);
    }
}

void primary_expr(void) {
    if (tok == TOK_NUM) {
        code_buf[code_idx] = 0xb8;
        unsigned int val = (unsigned int)token_num;
        k_memcpy((char *)(code_buf + code_idx + 1), (char *)&val, 4);
        code_idx = code_idx + 5;
        match(TOK_NUM);
    } else if (tok == TOK_STAR) {
        match(TOK_STAR);
        primary_expr();
        code_buf[code_idx] = 0x48;
        code_buf[code_idx + 1] = 0x89;
        code_buf[code_idx + 2] = 0xc6;
        code_buf[code_idx + 3] = 0x0f;
        code_buf[code_idx + 4] = 0xb6;
        code_buf[code_idx + 5] = 0x06;
        code_idx = code_idx + 6;
    } else if (tok == TOK_IDENT) {
        char name[256];
        k_strcpy(name, token_string);
        match(TOK_IDENT);

        if (tok == TOK_LPAREN) {
            match(TOK_LPAREN);

            if (k_strcmp(name, "syscall") == 0) {
                expression();
                code_buf[code_idx] = 0x50;
                code_idx = code_idx + 1;

                match(TOK_COMMA);
                expression();
                code_buf[code_idx] = 0x50;
                code_idx = code_idx + 1;

                match(TOK_COMMA);
                expression();
                code_buf[code_idx] = 0x50;
                code_idx = code_idx + 1;

                match(TOK_COMMA);
                expression();
                code_buf[code_idx] = 0x50;
                code_idx = code_idx + 1;

                match(TOK_RPAREN);

                code_buf[code_idx] = 0x5a;
                code_buf[code_idx + 1] = 0x5e;
                code_buf[code_idx + 2] = 0x5f;
                code_buf[code_idx + 3] = 0x58;
                code_buf[code_idx + 4] = 0x0f;
                code_buf[code_idx + 5] = 0x05;
                code_idx = code_idx + 6;
                return;
            }

            int arg_count = 0;
            if (tok != TOK_RPAREN) {
                expression();
                code_buf[code_idx] = 0x50;
                code_idx = code_idx + 1;
                arg_count = arg_count + 1;
                while (tok == TOK_COMMA) {
                    match(TOK_COMMA);
                    expression();
                    code_buf[code_idx] = 0x50;
                    code_idx = code_idx + 1;
                    arg_count = arg_count + 1;
                }
            }
            match(TOK_RPAREN);

            if (arg_count == 3) {
                code_buf[code_idx] = 0x5a;
                code_buf[code_idx + 1] = 0x5e;
                code_buf[code_idx + 2] = 0x5f;
                code_idx = code_idx + 3;
            } else if (arg_count == 2) {
                code_buf[code_idx] = 0x5e;
                code_buf[code_idx + 1] = 0x5f;
                code_idx = code_idx + 2;
            } else if (arg_count == 1) {
                code_buf[code_idx] = 0x5f;
                code_idx = code_idx + 1;
            }

            code_buf[code_idx] = 0xe8;
            int patch_pos = code_idx + 1;
            int zero = 0;
            k_memcpy((char *)(code_buf + code_idx + 1), (char *)&zero, 4);
            code_idx = code_idx + 5;

            k_strcpy(patch_names + patch_count * 32, name);
            patch_addresses[patch_count] = patch_pos;
            patch_count = patch_count + 1;
        } else {
            int local_offset = lookup_local(name);
            if (local_offset != 0) {
                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0x8b;
                code_buf[code_idx + 2] = 0x45;
                code_buf[code_idx + 3] = (unsigned char)local_offset;
                code_idx = code_idx + 4;

                if (tok == TOK_LBRACKET) {
                    match(TOK_LBRACKET);
                    code_buf[code_idx] = 0x50;
                    code_idx = code_idx + 1;

                    expression();

                    code_buf[code_idx] = 0x5e;
                    code_buf[code_idx + 1] = 0x48;
                    code_buf[code_idx + 2] = 0x01;
                    code_buf[code_idx + 3] = 0xc6;
                    code_buf[code_idx + 4] = 0x0f;
                    code_buf[code_idx + 5] = 0xb6;
                    code_buf[code_idx + 6] = 0x06;
                    code_idx = code_idx + 7;
                    match(TOK_RBRACKET);
                }
            } else {
                int global_offset = lookup_global(name);
                if (global_offset == 0 - 1) {
                    print_str("Error: Undefined variable ");
                    print_str(name);
                    print_str("\n");
                    sys_exit(1);
                }

                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0xbe;
                int patch_pos = code_idx + 2;
                unsigned long dummy_offset = (unsigned long)global_offset;
                k_memcpy((char *)(code_buf + code_idx + 2), (char *)&dummy_offset, 8);
                code_idx = code_idx + 10;

                val_patch_addresses[val_patch_count] = patch_pos;
                val_patch_offsets[val_patch_count] = global_offset;
                val_patch_count = val_patch_count + 1;

                if (tok == TOK_LBRACKET) {
                    match(TOK_LBRACKET);
                    code_buf[code_idx] = 0x56;
                    code_idx = code_idx + 1;

                    expression();

                    code_buf[code_idx] = 0x5e;
                    code_buf[code_idx + 1] = 0x48;
                    code_buf[code_idx + 2] = 0x01;
                    code_buf[code_idx + 3] = 0xc6;
                    code_buf[code_idx + 4] = 0x0f;
                    code_buf[code_idx + 5] = 0xb6;
                    code_buf[code_idx + 6] = 0x06;
                    code_idx = code_idx + 7;
                    match(TOK_RBRACKET);
                } else {
                    code_buf[code_idx] = 0x48;
                    code_buf[code_idx + 1] = 0x8b;
                    code_buf[code_idx + 2] = 0x06;
                    code_idx = code_idx + 3;
                }
            }
        }
    } else if (tok == TOK_LPAREN) {
        match(TOK_LPAREN);
        expression();
        match(TOK_RPAREN);
    } else {
        print_str("Error: Invalid primary expression, got token ");
        print_num(tok);
        print_str("\n");
        sys_exit(1);
    }
}

void mul_expr(void) {
    primary_expr();
    while (tok == TOK_STAR || tok == TOK_SLASH) {
        int op = tok;
        match(op);
        code_buf[code_idx] = 0x50;
        code_idx = code_idx + 1;
        primary_expr();
        code_buf[code_idx] = 0x59;
        code_idx = code_idx + 1;

        if (op == TOK_STAR) {
            code_buf[code_idx] = 0x0f;
            code_buf[code_idx + 1] = 0xaf;
            code_buf[code_idx + 2] = 0xc1;
            code_idx = code_idx + 3;
        } else {
            code_buf[code_idx] = 0x91;
            code_buf[code_idx + 1] = 0x99;
            code_buf[code_idx + 2] = 0xf7;
            code_buf[code_idx + 3] = 0xf9;
            code_idx = code_idx + 4;
        }
    }
}

void add_expr(void) {
    mul_expr();
    while (tok == TOK_PLUS || tok == TOK_MINUS) {
        int op = tok;
        match(op);
        code_buf[code_idx] = 0x50;
        code_idx = code_idx + 1;
        mul_expr();
        code_buf[code_idx] = 0x59;
        code_idx = code_idx + 1;

        if (op == TOK_PLUS) {
            code_buf[code_idx] = 0x03;
            code_buf[code_idx + 1] = 0xc1;
            code_idx = code_idx + 2;
        } else {
            code_buf[code_idx] = 0x91;
            code_buf[code_idx + 1] = 0x2b;
            code_buf[code_idx + 2] = 0xc1;
            code_idx = code_idx + 3;
        }
    }
}

void expression(void) {
    add_expr();
    while (tok == TOK_LT || tok == TOK_GT || tok == TOK_LEQ || tok == TOK_GEQ || tok == TOK_EQ ||
           tok == TOK_NEQ || tok == TOK_AND || tok == TOK_OR) {
        int op = tok;
        match(op);
        code_buf[code_idx] = 0x50;
        code_idx = code_idx + 1;
        add_expr();
        code_buf[code_idx] = 0x59;
        code_idx = code_idx + 1;

        if (op == TOK_AND) {
            code_buf[code_idx] = 0x85;
            code_buf[code_idx + 1] = 0xc9;
            code_buf[code_idx + 2] = 0x0f;
            code_buf[code_idx + 3] = 0x95;
            code_buf[code_idx + 4] = 0xc1;
            code_buf[code_idx + 5] = 0x85;
            code_buf[code_idx + 6] = 0xc0;
            code_buf[code_idx + 7] = 0x0f;
            code_buf[code_idx + 8] = 0x95;
            code_buf[code_idx + 9] = 0xc0;
            code_buf[code_idx + 10] = 0x22;
            code_buf[code_idx + 11] = 0xc1;
            code_buf[code_idx + 12] = 0x0f;
            code_buf[code_idx + 13] = 0xb6;
            code_buf[code_idx + 14] = 0xc0;
            code_idx = code_idx + 15;
        } else if (op == TOK_OR) {
            code_buf[code_idx] = 0x09;
            code_buf[code_idx + 1] = 0xc1;
            code_buf[code_idx + 2] = 0x85;
            code_buf[code_idx + 3] = 0xc0;
            code_buf[code_idx + 4] = 0x0f;
            code_buf[code_idx + 5] = 0x95;
            code_buf[code_idx + 6] = 0xc0;
            code_buf[code_idx + 7] = 0x0f;
            code_buf[code_idx + 8] = 0xb6;
            code_buf[code_idx + 9] = 0xc0;
            code_idx = code_idx + 10;
        } else {
            code_buf[code_idx] = 0x39;
            code_buf[code_idx + 1] = 0xc1;
            code_idx = code_idx + 2;

            if (op == TOK_LT) {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x9c;
                code_buf[code_idx + 2] = 0xc0;
            } else if (op == TOK_GT) {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x9f;
                code_buf[code_idx + 2] = 0xc0;
            } else if (op == TOK_LEQ) {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x9e;
                code_buf[code_idx + 2] = 0xc0;
            } else if (op == TOK_GEQ) {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x9d;
                code_buf[code_idx + 2] = 0xc0;
            } else if (op == TOK_EQ) {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x94;
                code_buf[code_idx + 2] = 0xc0;
            } else {
                code_buf[code_idx] = 0x0f;
                code_buf[code_idx + 1] = 0x95;
                code_buf[code_idx + 2] = 0xc0;
            }
            code_buf[code_idx + 3] = 0x0f;
            code_buf[code_idx + 4] = 0xb6;
            code_buf[code_idx + 5] = 0xc0;
            code_idx = code_idx + 6;
        }
    }
}

void block(void) {
    while (tok != TOK_RBRACE && tok != TOK_EOF) {
        statement();
    }
}

void statement(void) {
    if (tok == TOK_LBRACE) {
        match(TOK_LBRACE);
        block();
        match(TOK_RBRACE);
    } else if (tok == TOK_INT || tok == TOK_CHAR) {
        int type = tok;
        match(type);

        int is_ptr = 0;
        if (tok == TOK_STAR) {
            match(TOK_STAR);
            is_ptr = 1;
        }
        (void)is_ptr;

        char var_name[256];
        k_strcpy(var_name, token_string);
        match(TOK_IDENT);

        int offset = add_local(var_name);
        if (tok == TOK_ASSIGN) {
            match(TOK_ASSIGN);
            expression();
        } else {
            code_buf[code_idx] = 0xb8;
            int zero = 0;
            k_memcpy((char *)(code_buf + code_idx + 1), (char *)&zero, 4);
            code_idx = code_idx + 5;
        }
        code_buf[code_idx] = 0x48;
        code_buf[code_idx + 1] = 0x89;
        code_buf[code_idx + 2] = 0x45;
        code_buf[code_idx + 3] = (unsigned char)offset;
        code_idx = code_idx + 4;
        match(TOK_SEMICOLON);
    } else if (tok == TOK_STAR) {
        match(TOK_STAR);
        char var_name[256];
        k_strcpy(var_name, token_string);
        match(TOK_IDENT);

        int offset = lookup_local(var_name);
        if (offset != 0) {
            code_buf[code_idx] = 0x48;
            code_buf[code_idx + 1] = 0x8b;
            code_buf[code_idx + 2] = 0x75;
            code_buf[code_idx + 3] = (unsigned char)offset;
            code_idx = code_idx + 4;
        } else {
            int global_offset = lookup_global(var_name);
            if (global_offset == 0 - 1) {
                print_str("Error: Undefined variable ");
                print_str(var_name);
                print_str("\n");
                sys_exit(1);
            }
            code_buf[code_idx] = 0x48;
            code_buf[code_idx + 1] = 0xbe;
            int patch_pos = code_idx + 2;
            unsigned long dummy_offset = (unsigned long)global_offset;
            k_memcpy((char *)(code_buf + code_idx + 2), (char *)&dummy_offset, 8);
            code_idx = code_idx + 10;

            val_patch_addresses[val_patch_count] = patch_pos;
            val_patch_offsets[val_patch_count] = global_offset;
            val_patch_count = val_patch_count + 1;

            code_buf[code_idx] = 0x48;
            code_buf[code_idx + 1] = 0x8b;
            code_buf[code_idx + 2] = 0x36;
            code_idx = code_idx + 3;
        }

        code_buf[code_idx] = 0x56;
        code_idx = code_idx + 1;

        match(TOK_ASSIGN);
        expression();
        match(TOK_SEMICOLON);

        code_buf[code_idx] = 0x5e;
        code_buf[code_idx + 1] = 0x88;
        code_buf[code_idx + 2] = 0x06;
        code_idx = code_idx + 3;
    } else if (tok == TOK_IDENT) {
        char name[256];
        k_strcpy(name, token_string);
        match(TOK_IDENT);

        if (tok == TOK_LBRACKET) {
            match(TOK_LBRACKET);

            int local_offset = lookup_local(name);
            if (local_offset != 0) {
                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0x8b;
                code_buf[code_idx + 2] = 0x75;
                code_buf[code_idx + 3] = (unsigned char)local_offset;
                code_idx = code_idx + 4;
            } else {
                int global_offset = lookup_global(name);
                if (global_offset == 0 - 1) {
                    print_str("Error: Undefined variable ");
                    print_str(name);
                    print_str("\n");
                    sys_exit(1);
                }
                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0xbe;
                int patch_pos = code_idx + 2;
                unsigned long dummy_offset = (unsigned long)global_offset;
                k_memcpy((char *)(code_buf + code_idx + 2), (char *)&dummy_offset, 8);
                code_idx = code_idx + 10;

                val_patch_addresses[val_patch_count] = patch_pos;
                val_patch_offsets[val_patch_count] = global_offset;
                val_patch_count = val_patch_count + 1;
            }

            code_buf[code_idx] = 0x56;
            code_idx = code_idx + 1;

            expression();
            match(TOK_RBRACKET);

            code_buf[code_idx] = 0x5e;
            code_buf[code_idx + 1] = 0x48;
            code_buf[code_idx + 2] = 0x01;
            code_buf[code_idx + 3] = 0xc6;
            code_buf[code_idx + 4] = 0x56;
            code_idx = code_idx + 5;

            match(TOK_ASSIGN);
            expression();
            match(TOK_SEMICOLON);

            code_buf[code_idx] = 0x5e;
            code_buf[code_idx + 1] = 0x88;
            code_buf[code_idx + 2] = 0x06;
            code_idx = code_idx + 3;
        } else {
            int local_offset = lookup_local(name);
            if (local_offset != 0) {
                match(TOK_ASSIGN);
                expression();
                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0x89;
                code_buf[code_idx + 2] = 0x45;
                code_buf[code_idx + 3] = (unsigned char)local_offset;
                code_idx = code_idx + 4;
                match(TOK_SEMICOLON);
            } else {
                int global_offset = lookup_global(name);
                if (global_offset == 0 - 1) {
                    print_str("Error: Undefined variable ");
                    print_str(name);
                    print_str("\n");
                    sys_exit(1);
                }

                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0xbe;
                int patch_pos = code_idx + 2;
                unsigned long dummy_offset = (unsigned long)global_offset;
                k_memcpy((char *)(code_buf + code_idx + 2), (char *)&dummy_offset, 8);
                code_idx = code_idx + 10;

                val_patch_addresses[val_patch_count] = patch_pos;
                val_patch_offsets[val_patch_count] = global_offset;
                val_patch_count = val_patch_count + 1;

                code_buf[code_idx] = 0x56;
                code_idx = code_idx + 1;

                match(TOK_ASSIGN);
                expression();
                match(TOK_SEMICOLON);

                code_buf[code_idx] = 0x5e;
                code_buf[code_idx + 1] = 0x48;
                code_buf[code_idx + 2] = 0x89;
                code_buf[code_idx + 3] = 0x06;
                code_idx = code_idx + 4;
            }
        }
    } else if (tok == TOK_IF) {
        match(TOK_IF);
        match(TOK_LPAREN);
        expression();
        match(TOK_RPAREN);

        code_buf[code_idx] = 0x85;
        code_buf[code_idx + 1] = 0xc0;
        code_buf[code_idx + 2] = 0x0f;
        code_buf[code_idx + 3] = 0x84;
        int jz_offset_idx = code_idx + 4;
        code_idx = code_idx + 8;

        statement();

        if (tok == TOK_ELSE) {
            match(TOK_ELSE);
            code_buf[code_idx] = 0xe9;
            int jmp_offset_idx = code_idx + 1;
            code_idx = code_idx + 5;

            int else_offset = code_idx - (jz_offset_idx + 4);
            k_memcpy((char *)(code_buf + jz_offset_idx), (char *)&else_offset, 4);

            statement();

            int end_offset = code_idx - (jmp_offset_idx + 4);
            k_memcpy((char *)(code_buf + jmp_offset_idx), (char *)&end_offset, 4);
        } else {
            int else_offset = code_idx - (jz_offset_idx + 4);
            k_memcpy((char *)(code_buf + jz_offset_idx), (char *)&else_offset, 4);
        }
    } else if (tok == TOK_WHILE) {
        int start_addr = code_idx;
        match(TOK_WHILE);
        match(TOK_LPAREN);
        expression();
        match(TOK_RPAREN);

        code_buf[code_idx] = 0x85;
        code_buf[code_idx + 1] = 0xc0;
        code_buf[code_idx + 2] = 0x0f;
        code_buf[code_idx + 3] = 0x84;
        int jz_offset_idx = code_idx + 4;
        code_idx = code_idx + 8;

        statement();

        code_buf[code_idx] = 0xe9;
        int jump_back = start_addr - (code_idx + 5);
        k_memcpy((char *)(code_buf + code_idx + 1), (char *)&jump_back, 4);
        code_idx = code_idx + 5;

        int end_offset = code_idx - (jz_offset_idx + 4);
        k_memcpy((char *)(code_buf + jz_offset_idx), (char *)&end_offset, 4);
    } else if (tok == TOK_FOR) {
        match(TOK_FOR);
        match(TOK_LPAREN);
        if (tok != TOK_SEMICOLON) {
            statement();
        } else {
            match(TOK_SEMICOLON);
        }
        int cond_addr = code_idx;
        if (tok != TOK_SEMICOLON) {
            expression();
        } else {
            code_buf[code_idx] = 0xb8;
            int one = 1;
            k_memcpy((char *)(code_buf + code_idx + 1), (char *)&one, 4);
            code_idx = code_idx + 5;
        }
        match(TOK_SEMICOLON);

        code_buf[code_idx] = 0x85;
        code_buf[code_idx + 1] = 0xc0;
        code_buf[code_idx + 2] = 0x0f;
        code_buf[code_idx + 3] = 0x84;
        int jz_offset_idx = code_idx + 4;
        code_idx = code_idx + 8;

        code_buf[code_idx] = 0xe9;
        int jmp_to_body_idx = code_idx + 1;
        code_idx = code_idx + 5;

        int post_addr = code_idx;
        if (tok != TOK_RPAREN) {
            expression();
        }
        match(TOK_RPAREN);

        code_buf[code_idx] = 0xe9;
        int jump_cond = cond_addr - (code_idx + 5);
        k_memcpy((char *)(code_buf + code_idx + 1), (char *)&jump_cond, 4);
        code_idx = code_idx + 5;

        int body_addr = code_idx;
        int jump_body = body_addr - (jmp_to_body_idx + 4);
        k_memcpy((char *)(code_buf + jmp_to_body_idx), (char *)&jump_body, 4);

        statement();

        code_buf[code_idx] = 0xe9;
        int jump_post = post_addr - (code_idx + 5);
        k_memcpy((char *)(code_buf + code_idx + 1), (char *)&jump_post, 4);
        code_idx = code_idx + 5;

        int end_offset = code_idx - (jz_offset_idx + 4);
        k_memcpy((char *)(code_buf + jz_offset_idx), (char *)&end_offset, 4);
    } else if (tok == TOK_PRINTF) {
        match(TOK_PRINTF);
        match(TOK_LPAREN);

        int str_len = k_strlen(token_string) + 1;
        if (data_idx + str_len >= MAX_DATA_SIZE) {
            print_str("Error: Data segment overflow\n");
            sys_exit(1);
        }
        int str_offset = data_idx;
        k_memcpy((char *)(data_buf + data_idx), token_string, str_len);
        data_idx = data_idx + str_len;

        match(TOK_STRING);
        match(TOK_RPAREN);
        match(TOK_SEMICOLON);

        code_buf[code_idx] = 0x48;
        code_buf[code_idx + 1] = 0xbe;
        unsigned long temp_offset = (unsigned long)str_offset;
        k_memcpy((char *)(code_buf + code_idx + 2), (char *)&temp_offset, 8);

        val_patch_addresses[val_patch_count] = code_idx + 2;
        val_patch_offsets[val_patch_count] = str_offset;
        val_patch_count = val_patch_count + 1;

        code_idx = code_idx + 10;

        code_buf[code_idx] = 0x0f;
        code_buf[code_idx + 1] = 0xb6;
        code_buf[code_idx + 2] = 0x3e;
        code_buf[code_idx + 3] = 0x85;
        code_buf[code_idx + 4] = 0xff;
        code_buf[code_idx + 5] = 0x74;
        code_buf[code_idx + 6] = 0x0c;
        code_buf[code_idx + 7] = 0xb8;
        code_buf[code_idx + 8] = 0x01;
        code_buf[code_idx + 9] = 0x00;
        code_buf[code_idx + 10] = 0x00;
        code_buf[code_idx + 11] = 0x00;
        code_buf[code_idx + 12] = 0x0f;
        code_buf[code_idx + 13] = 0x05;
        code_buf[code_idx + 14] = 0x48;
        code_buf[code_idx + 15] = 0xff;
        code_buf[code_idx + 16] = 0xc6;
        code_buf[code_idx + 17] = 0xeb;
        code_buf[code_idx + 18] = 0xed;
        code_idx = code_idx + 19;
    } else if (tok == TOK_RETURN) {
        match(TOK_RETURN);
        expression();
        match(TOK_SEMICOLON);
    } else {
        print_str("Error: Unexpected token in statement: ");
        print_num(tok);
        print_str("\n");
        sys_exit(1);
    }
}

void compile_global_declarations(void) {
    tok = next_token();
    while (tok != TOK_EOF) {
        if (tok == TOK_INT || tok == TOK_VOID || tok == TOK_CHAR) {
            int type = tok;
            match(type);

            int is_ptr = 0;
            if (tok == TOK_STAR) {
                match(TOK_STAR);
                is_ptr = 1;
            }
            (void)is_ptr;

            char name[256];
            k_strcpy(name, token_string);
            match(TOK_IDENT);

            if (tok == TOK_LPAREN) {
                match(TOK_LPAREN);
                add_function(name, code_idx);
                local_count = 0;

                int param_count = 0;
                if (tok != TOK_RPAREN) {
                    int p_type = tok;
                    match(p_type);
                    if (p_type == TOK_VOID && tok == TOK_RPAREN) {
                    } else {
                        int p_is_ptr = 0;
                        if (tok == TOK_STAR) {
                            match(TOK_STAR);
                            p_is_ptr = 1;
                        }
                        (void)p_is_ptr;
                        char p_name[256];
                        k_strcpy(p_name, token_string);
                        match(TOK_IDENT);

                        add_local(p_name);
                        param_count = param_count + 1;

                        while (tok == TOK_COMMA) {
                            match(TOK_COMMA);
                            int next_p_type = tok;
                            match(next_p_type);
                            int next_p_is_ptr = 0;
                            if (tok == TOK_STAR) {
                                match(TOK_STAR);
                                next_p_is_ptr = 1;
                            }
                            (void)next_p_is_ptr;
                            char next_p_name[256];
                            k_strcpy(next_p_name, token_string);
                            match(TOK_IDENT);

                            add_local(next_p_name);
                            param_count = param_count + 1;
                        }
                    }
                }
                match(TOK_RPAREN);

                code_buf[code_idx] = 0x55;
                code_buf[code_idx + 1] = 0x48;
                code_buf[code_idx + 2] = 0x89;
                code_buf[code_idx + 3] = 0xe5;
                code_buf[code_idx + 4] = 0x48;
                code_buf[code_idx + 5] = 0x81;
                code_buf[code_idx + 6] = 0xec;
                code_buf[code_idx + 7] = 0x80;
                code_buf[code_idx + 8] = 0x00;
                code_buf[code_idx + 9] = 0x00;
                code_buf[code_idx + 10] = 0x00;
                code_idx = code_idx + 11;

                if (param_count >= 1) {
                    code_buf[code_idx] = 0x48;
                    code_buf[code_idx + 1] = 0x89;
                    code_buf[code_idx + 2] = 0x7d;
                    code_buf[code_idx + 3] = 0xf8;
                    code_idx = code_idx + 4;
                }
                if (param_count >= 2) {
                    code_buf[code_idx] = 0x48;
                    code_buf[code_idx + 1] = 0x89;
                    code_buf[code_idx + 2] = 0x75;
                    code_buf[code_idx + 3] = 0xf0;
                    code_idx = code_idx + 4;
                }
                if (param_count >= 3) {
                    code_buf[code_idx] = 0x48;
                    code_buf[code_idx + 1] = 0x89;
                    code_buf[code_idx + 2] = 0x55;
                    code_buf[code_idx + 3] = 0xe8;
                    code_idx = code_idx + 4;
                }
                if (param_count >= 4) {
                    code_buf[code_idx] = 0x48;
                    code_buf[code_idx + 1] = 0x89;
                    code_buf[code_idx + 2] = 0x4d;
                    code_buf[code_idx + 3] = 0xe0;
                    code_idx = code_idx + 4;
                }

                match(TOK_LBRACE);
                block();
                match(TOK_RBRACE);

                code_buf[code_idx] = 0x48;
                code_buf[code_idx + 1] = 0x89;
                code_buf[code_idx + 2] = 0xec;
                code_buf[code_idx + 3] = 0x5d;
                code_idx = code_idx + 4;

                if (k_strcmp(name, "main") == 0) {
                    code_buf[code_idx] = 0xb8;
                    code_buf[code_idx + 1] = 0x02;
                    code_buf[code_idx + 2] = 0x00;
                    code_buf[code_idx + 3] = 0x00;
                    code_buf[code_idx + 4] = 0x00;
                    code_buf[code_idx + 5] = 0x0f;
                    code_buf[code_idx + 6] = 0x05;
                    code_buf[code_idx + 7] = 0xeb;
                    code_buf[code_idx + 8] = 0xfe;
                    code_idx = code_idx + 9;
                } else {
                    code_buf[code_idx] = 0xc3;
                    code_idx = code_idx + 1;
                }
            } else {
                if (tok == TOK_LBRACKET) {
                    match(TOK_LBRACKET);
                    int size = token_num;
                    match(TOK_NUM);
                    match(TOK_RBRACKET);
                    add_global(name, size);
                } else {
                    add_global(name, 8);
                }
                match(TOK_SEMICOLON);
            }
        } else {
            tok = next_token();
        }
    }
}
