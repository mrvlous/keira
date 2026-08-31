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

#include "codegen.h"
#include "common.h"
#include "lexer.h"
#include "symbols.h"

#include <syscall.h>

void match(int expected) {
    if (tok == expected) {
        tok = next_token();
    } else {
        print_str("Error on line ");
        print_num(line_num);
        print_str(": Expected ");
        print_str(token_name(expected));
        print_str(", got ");
        print_str(token_name(tok));
        print_str(" ('");
        print_str(token_string);
        print_str("')\n");
        sys_exit(1);
    }
}

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

        emit_syscall_stub();
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

                emit_syscall_stub();
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
            emit_u8(0x48);
            emit_u8(0x01);
            emit_u8(0xc8); /* add rax, rcx (rax = base + index) */
            emit_deref(1); /* byte deref */
        } else if (tok == TOK_INC) {
            match(TOK_INC);
            /* Post-increment: rax holds original value, but we increment variable if direct local
             */
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

/* Multiplicative: *, /, % */
void mul_expr(void) {
    unary_expr();
    while (tok == TOK_STAR || tok == TOK_SLASH || tok == TOK_MOD) {
        int op = tok;
        match(op);
        emit_push_rax();
        unary_expr();
        emit_pop_rcx(); /* rcx = LHS, rax = RHS */

        if (op == TOK_STAR)
            emit_imul();
        else if (op == TOK_SLASH)
            emit_idiv();
        else
            emit_imod();
    }
}

/* Additive: +, - */
void add_expr(void) {
    mul_expr();
    while (tok == TOK_PLUS || tok == TOK_MINUS) {
        int op = tok;
        match(op);
        emit_push_rax();
        mul_expr();
        emit_pop_rcx(); /* rcx = LHS, rax = RHS */

        if (op == TOK_PLUS)
            emit_add();
        else
            emit_sub();
    }
}

/* Shift: <<, >> */
void shift_expr(void) {
    add_expr();
    while (tok == TOK_SHL || tok == TOK_SHR) {
        int op = tok;
        match(op);
        emit_push_rax();
        add_expr();
        emit_pop_rcx();

        if (op == TOK_SHL)
            emit_shl();
        else
            emit_shr();
    }
}

/* Relational: <, >, <=, >= */
void rel_expr(void) {
    shift_expr();
    while (tok == TOK_LT || tok == TOK_GT || tok == TOK_LEQ || tok == TOK_GEQ) {
        int op = tok;
        match(op);
        emit_push_rax();
        shift_expr();
        emit_pop_rcx();

        emit_cmp_set(op);
    }
}

/* Equality: ==, != */
void eq_expr(void) {
    rel_expr();
    while (tok == TOK_EQ || tok == TOK_NEQ) {
        int op = tok;
        match(op);
        emit_push_rax();
        rel_expr();
        emit_pop_rcx();

        emit_cmp_set(op);
    }
}

/* Bitwise AND: & */
void bit_and_expr(void) {
    eq_expr();
    while (tok == TOK_AMP) {
        match(TOK_AMP);
        emit_push_rax();
        eq_expr();
        emit_pop_rcx();
        emit_bit_and();
    }
}

/* Bitwise XOR: ^ */
void bit_xor_expr(void) {
    bit_and_expr();
    while (tok == TOK_CARET) {
        match(TOK_CARET);
        emit_push_rax();
        bit_and_expr();
        emit_pop_rcx();
        emit_bit_xor();
    }
}

/* Bitwise OR: | */
void bit_or_expr(void) {
    bit_xor_expr();
    while (tok == TOK_PIPE) {
        match(TOK_PIPE);
        emit_push_rax();
        bit_xor_expr();
        emit_pop_rcx();
        emit_bit_or();
    }
}

/* Logical AND: && */
void log_and_expr(void) {
    bit_or_expr();
    while (tok == TOK_AND) {
        match(TOK_AND);
        emit_push_rax();
        bit_or_expr();
        emit_pop_rcx();
        emit_log_and();
    }
}

/* Logical OR: || */
void log_or_expr(void) {
    log_and_expr();
    while (tok == TOK_OR) {
        match(TOK_OR);
        emit_push_rax();
        log_and_expr();
        emit_pop_rcx();
        emit_log_or();
    }
}

/* Assignment Expressions (=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=) */
void assignment_expr(void) {
    log_or_expr();
}

void expression(void) {
    assignment_expr();
}

/* Statements & Control Flow */
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
            emit_pop_rcx(); /* rcx = base */
            emit_u8(0x48);
            emit_u8(0x01);
            emit_u8(0xc8);   /* add rax, rcx (effective address) */
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
                if (tok == TOK_ASSIGN) {
                    match(TOK_ASSIGN);
                    assignment_expr();
                    int loc = lookup_local(var_name);
                    if (loc != 0)
                        emit_store_local(loc, 8);
                } else if (tok == TOK_INC) {
                    match(TOK_INC);
                    int loc = lookup_local(var_name);
                    if (loc != 0)
                        emit_inc_local(loc, 0, 0);
                } else if (tok == TOK_DEC) {
                    match(TOK_DEC);
                    int loc = lookup_local(var_name);
                    if (loc != 0)
                        emit_inc_local(loc, 0, 1);
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

        emit_printf_stub(fmt_offset, arg_count);
    } else if (tok == TOK_RETURN) {
        match(TOK_RETURN);
        if (tok != TOK_SEMICOLON) {
            assignment_expr();
        } else {
            emit_load_imm(0);
        }
        match(TOK_SEMICOLON);
        emit_func_epilogue(0);
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

/* Global Declarations: Variables, Arrays, and Function Definitions */
void compile_global_declarations(void) {
    tok = next_token();
    while (tok != TOK_EOF) {
        if (tok == TOK_INT || tok == TOK_CHAR || tok == TOK_VOID || tok == TOK_SHORT ||
            tok == TOK_LONG || tok == TOK_UNSIGNED) {
            int type_tok = tok;
            match(type_tok);

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
                /* Function Definition */
                match(TOK_LPAREN);
                add_function(name, code_idx);
                clear_locals();

                int param_count = 0;
                if (tok != TOK_RPAREN) {
                    if (tok == TOK_VOID && *(src_ptr) == ')') {
                        match(TOK_VOID);
                    } else {
                        int p_type = tok;
                        match(p_type);
                        int p_is_ptr = 0;
                        if (tok == TOK_STAR) {
                            match(TOK_STAR);
                            p_is_ptr = 1;
                        }
                        (void)p_is_ptr;
                        char p_name[256];
                        k_strcpy(p_name, token_string);
                        match(TOK_IDENT);

                        add_local(p_name, 8);
                        param_count++;

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

                            add_local(next_p_name, 8);
                            param_count++;
                        }
                    }
                }
                match(TOK_RPAREN);

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

                int is_main = (k_strcmp(name, "main") == 0);
                emit_func_epilogue(is_main);
            } else if (tok == TOK_LBRACKET) {
                /* Global Array: char buf[1024]; */
                match(TOK_LBRACKET);
                int size = (int)token_num;
                match(TOK_NUM);
                match(TOK_RBRACKET);
                add_global(name, size);
                match(TOK_SEMICOLON);
            } else {
                /* Global Variable: int x; */
                add_global(name, 8);
                match(TOK_SEMICOLON);
            }
        } else {
            tok = next_token();
        }
    }
}
