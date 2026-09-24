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
#include "lexer.h"
#include "parser.h"

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
