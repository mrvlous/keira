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

void statement(void) {
    if (tok == TOK_LBRACE) {
        match(TOK_LBRACE);
        block();
        match(TOK_RBRACE);
    } else if (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_SHORT || tok == TOK_LONG ||
               tok == TOK_UNSIGNED || tok == TOK_VOID) {
        /* Local variable declaration: int x = 5, y = 10; */
        int type_tok = tok;
        match(type_tok);

        int is_ptr = 0;
        if (tok == TOK_STAR) {
            match(TOK_STAR);
            is_ptr = 1;
        }
        (void)is_ptr;

        char var_name[256];
        k_strcpy(var_name, token_string);
        match(TOK_IDENT);

        int offset = add_local(var_name, 8);
        if (tok == TOK_ASSIGN) {
            match(TOK_ASSIGN);
            assignment_expr();
        } else {
            emit_load_imm(0);
        }
        emit_store_local(offset, 8);

        while (tok == TOK_COMMA) {
            match(TOK_COMMA);
            int next_is_ptr = 0;
            if (tok == TOK_STAR) {
                match(TOK_STAR);
                next_is_ptr = 1;
            }
            (void)next_is_ptr;
            char next_name[256];
            k_strcpy(next_name, token_string);
            match(TOK_IDENT);

            int next_offset = add_local(next_name, 8);
            if (tok == TOK_ASSIGN) {
                match(TOK_ASSIGN);
                assignment_expr();
            } else {
                emit_load_imm(0);
            }
            emit_store_local(next_offset, 8);
        }
        match(TOK_SEMICOLON);
    } else if (tok == TOK_STAR) {
        /* Pointer write: *ptr = expr; */
        match(TOK_STAR);
        char var_name[256];
        k_strcpy(var_name, token_string);
        match(TOK_IDENT);

        int offset = lookup_local(var_name);
        if (offset != 0) {
            emit_load_local(offset, 8);
        } else {
            int glob = lookup_global(var_name);
            if (glob == -1) {
                error_msg("Undefined variable in pointer assignment");
                sys_exit(1);
            }
            emit_load_global(glob, 8);
        }
        emit_push_rax(); /* save target pointer */

        match(TOK_ASSIGN);
        assignment_expr();
        match(TOK_SEMICOLON);

        emit_pop_rdx(); /* rdx = pointer */
        emit_store_deref(8);
    } else if (tok == TOK_IDENT) {
        char name[256];
        k_strcpy(name, token_string);
        match(TOK_IDENT);

        if (tok == TOK_LBRACKET) {
            /* Array element assignment: arr[idx] = expr; */
            match(TOK_LBRACKET);
            int loc = lookup_local(name);
            if (loc != 0) {
                emit_load_local(loc, 8);
            } else {
                int glob = lookup_global(name);
                if (glob == -1) {
                    error_msg("Undefined array identifier");
                    sys_exit(1);
                }
                emit_addr_global(glob);
            }
            emit_push_rax();   /* base */
            assignment_expr(); /* index */
            match(TOK_RBRACKET);

            /* index in rax, base in stack */
            emit_pop_rcx();  /* rcx = base */
            emit_add();      /* add rax, rcx (effective address, dual-arch safe) */
            emit_push_rax(); /* push effective address */

            match(TOK_ASSIGN);
            assignment_expr();
            match(TOK_SEMICOLON);

            emit_pop_rdx();      /* rdx = address */
            emit_store_deref(1); /* byte write */
        } else if (tok == TOK_ASSIGN || tok == TOK_ADD_ASSIGN || tok == TOK_SUB_ASSIGN ||
                   tok == TOK_MUL_ASSIGN || tok == TOK_DIV_ASSIGN || tok == TOK_MOD_ASSIGN ||
                   tok == TOK_AND_ASSIGN || tok == TOK_OR_ASSIGN || tok == TOK_XOR_ASSIGN ||
                   tok == TOK_SHL_ASSIGN || tok == TOK_SHR_ASSIGN) {
            int assign_op = tok;
            match(assign_op);

            int loc = lookup_local(name);
            int glob = (loc == 0) ? lookup_global(name) : -1;

            if (loc == 0 && glob == -1) {
                print_str("Error on line ");
                print_num(line_num);
                print_str(": Undefined variable '");
                print_str(name);
                print_str("'\n");
                sys_exit(1);
            }

            if (assign_op == TOK_ASSIGN) {
                assignment_expr();
            } else {
                /* Compound assignment: load previous value, evaluate RHS, apply operator */
                if (loc != 0) {
                    emit_load_local(loc, 8);
                } else {
                    emit_load_global(glob, 8);
                }
                emit_push_rax();   /* LHS */
                assignment_expr(); /* RHS */
                emit_pop_rcx();    /* rcx = LHS, rax = RHS */

                switch (assign_op) {
                case TOK_ADD_ASSIGN:
                    emit_add();
                    break;
                case TOK_SUB_ASSIGN:
                    emit_sub();
                    break;
                case TOK_MUL_ASSIGN:
                    emit_imul();
                    break;
                case TOK_DIV_ASSIGN:
                    emit_idiv();
                    break;
                case TOK_MOD_ASSIGN:
                    emit_imod();
                    break;
                case TOK_AND_ASSIGN:
                    emit_bit_and();
                    break;
                case TOK_OR_ASSIGN:
                    emit_bit_or();
                    break;
                case TOK_XOR_ASSIGN:
                    emit_bit_xor();
                    break;
                case TOK_SHL_ASSIGN:
                    emit_shl();
                    break;
                case TOK_SHR_ASSIGN:
                    emit_shr();
                    break;
                default:
                    break;
                }
            }

            if (loc != 0) {
                emit_store_local(loc, 8);
            } else {
                emit_store_global(glob, 8);
            }
            match(TOK_SEMICOLON);
        } else if (tok == TOK_INC || tok == TOK_DEC) {
            int is_dec = (tok == TOK_DEC);
            match(tok);
            int loc = lookup_local(name);
            if (loc != 0) {
                emit_inc_local(loc, 0, is_dec);
            } else {
                int glob = lookup_global(name);
                if (glob != -1) {
                    emit_inc_global(glob, 0, is_dec);
                } else {
                    print_str("Error on line ");
                    print_num(line_num);
                    print_str(": Undefined variable in increment/decrement '");
                    print_str(name);
                    print_str("'\n");
                    sys_exit(1);
                }
            }
            match(TOK_SEMICOLON);
        } else if (tok == TOK_LPAREN) {
            /* Function call as statement */
            match(TOK_LPAREN);
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
            match(TOK_SEMICOLON);
        } else {
            match(TOK_SEMICOLON);
        }
    } else if (tok == TOK_IF) {
        match(TOK_IF);
        match(TOK_LPAREN);
        assignment_expr();
        match(TOK_RPAREN);

        int jz_patch = emit_jz_forward();
        statement();

        if (tok == TOK_ELSE) {
            match(TOK_ELSE);
            int jmp_patch = emit_jmp_forward();
            patch_jump(jz_patch, code_idx);
            statement();
            patch_jump(jmp_patch, code_idx);
        } else {
            patch_jump(jz_patch, code_idx);
        }
    } else if (tok == TOK_WHILE) {
        match(TOK_WHILE);
        int loop_start = code_idx;
        push_loop(loop_start);

        match(TOK_LPAREN);
        assignment_expr();
        match(TOK_RPAREN);

        int exit_jz = emit_jz_forward();
        statement();

        emit_jmp_backward(loop_start);
        patch_jump(exit_jz, code_idx);
        pop_loop(code_idx);
    } else if (tok == TOK_DO) {
        match(TOK_DO);
        int loop_start = code_idx;
        push_loop(loop_start);

        statement();

        match(TOK_WHILE);
        match(TOK_LPAREN);
        assignment_expr();
        match(TOK_RPAREN);
        match(TOK_SEMICOLON);

        int loop_jnz = emit_jnz_forward();
        patch_jump(loop_jnz, loop_start);
        pop_loop(code_idx);
    } else if (tok == TOK_FOR) {
        match(TOK_FOR);
        match(TOK_LPAREN);

        /* Initializer */
        if (tok != TOK_SEMICOLON) {
            statement();
        } else {
            match(TOK_SEMICOLON);
        }

        int cond_addr = code_idx;

        /* Condition */
        if (tok != TOK_SEMICOLON) {
            assignment_expr();
        } else {
            emit_load_imm(1);
        }
        match(TOK_SEMICOLON);

        int exit_jz = emit_jz_forward();
        int jmp_body = emit_jmp_forward();

        int post_addr = code_idx;
        push_loop(post_addr);

        /* Post-expression */
        if (tok != TOK_RPAREN) {
            if (tok == TOK_IDENT) {
                char var_name[256];
                k_strcpy(var_name, token_string);
                match(TOK_IDENT);
                int loc = lookup_local(var_name);
                int glob = (loc == 0) ? lookup_global(var_name) : -1;

                if (tok == TOK_INC) {
                    match(TOK_INC);
                    if (loc != 0) {
                        emit_inc_local(loc, 0, 0);
                    } else if (glob != -1) {
                        emit_inc_global(glob, 0, 0);
                    }
                } else if (tok == TOK_DEC) {
                    match(TOK_DEC);
                    if (loc != 0) {
                        emit_inc_local(loc, 0, 1);
                    } else if (glob != -1) {
                        emit_inc_global(glob, 0, 1);
                    }
                } else if (tok == TOK_ASSIGN || tok == TOK_ADD_ASSIGN || tok == TOK_SUB_ASSIGN ||
                           tok == TOK_MUL_ASSIGN || tok == TOK_DIV_ASSIGN ||
                           tok == TOK_MOD_ASSIGN || tok == TOK_AND_ASSIGN || tok == TOK_OR_ASSIGN ||
                           tok == TOK_XOR_ASSIGN || tok == TOK_SHL_ASSIGN ||
                           tok == TOK_SHR_ASSIGN) {
                    int assign_op = tok;
                    match(assign_op);

                    if (assign_op == TOK_ASSIGN) {
                        assignment_expr();
                    } else {
                        if (loc != 0) {
                            emit_load_local(loc, 8);
                        } else if (glob != -1) {
                            emit_load_global(glob, 8);
                        }
                        emit_push_rax();
                        assignment_expr();
                        emit_pop_rcx();

                        switch (assign_op) {
                        case TOK_ADD_ASSIGN:
                            emit_add();
                            break;
                        case TOK_SUB_ASSIGN:
                            emit_sub();
                            break;
                        case TOK_MUL_ASSIGN:
                            emit_imul();
                            break;
                        case TOK_DIV_ASSIGN:
                            emit_idiv();
                            break;
                        case TOK_MOD_ASSIGN:
                            emit_imod();
                            break;
                        case TOK_AND_ASSIGN:
                            emit_bit_and();
                            break;
                        case TOK_OR_ASSIGN:
                            emit_bit_or();
                            break;
                        case TOK_XOR_ASSIGN:
                            emit_bit_xor();
                            break;
                        case TOK_SHL_ASSIGN:
                            emit_shl();
                            break;
                        case TOK_SHR_ASSIGN:
                            emit_shr();
                            break;
                        default:
                            break;
                        }
                    }

                    if (loc != 0) {
                        emit_store_local(loc, 8);
                    } else if (glob != -1) {
                        emit_store_global(glob, 8);
                    }
                }
            } else {
                assignment_expr();
            }
        }
        match(TOK_RPAREN);

        emit_jmp_backward(cond_addr);
        patch_jump(jmp_body, code_idx);

        statement();

        emit_jmp_backward(post_addr);
        patch_jump(exit_jz, code_idx);
        pop_loop(code_idx);
    } else if (tok == TOK_BREAK) {
        match(TOK_BREAK);
        int jmp_patch = emit_jmp_forward();
        add_loop_break_patch(jmp_patch);
        match(TOK_SEMICOLON);
    } else if (tok == TOK_CONTINUE) {
        match(TOK_CONTINUE);
        int jmp_patch = emit_jmp_forward();
        add_loop_continue_patch(jmp_patch);
        match(TOK_SEMICOLON);
    } else if (tok == TOK_PRINTF) {
        match(TOK_PRINTF);
        match(TOK_LPAREN);

        int str_len = k_strlen(token_string) + 1;
        if (data_idx + str_len >= MAX_DATA_SIZE) {
            error_msg("Data segment overflow for printf format");
            sys_exit(1);
        }
        int fmt_offset = data_idx;
        k_memcpy((char *)(data_buf + data_idx), token_string, str_len);
        data_idx += str_len;

        match(TOK_STRING);

        int arg_count = 0;
        while (tok == TOK_COMMA) {
            match(TOK_COMMA);
            assignment_expr();
            emit_push_rax();
            arg_count++;
        }
        match(TOK_RPAREN);
        match(TOK_SEMICOLON);

        emit_printf_trampoline(fmt_offset, arg_count);
    } else if (tok == TOK_RETURN) {
        match(TOK_RETURN);
        if (tok != TOK_SEMICOLON) {
            assignment_expr();
        } else {
            emit_load_imm(0);
        }
        match(TOK_SEMICOLON);
        emit_func_epilogue(current_is_main);
    } else {
        /* Expression statement or empty statement */
        if (tok == TOK_SEMICOLON) {
            match(TOK_SEMICOLON);
        } else {
            assignment_expr();
            match(TOK_SEMICOLON);
        }
    }
}
