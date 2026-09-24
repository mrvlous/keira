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

/* Comparison and Relational Sets (rcx = LHS, rax = RHS) */
void emit_cmp_set(int op_tok) {
    /* cmp rcx/ecx, rax/eax */
    REX_W();
    emit_u8(0x39);
    emit_u8(0xc1);

    switch (op_tok) {
    case TOK_EQ:
        emit_u8(0x0f);
        emit_u8(0x94);
        emit_u8(0xc0); /* sete al */
        break;
    case TOK_NEQ:
        emit_u8(0x0f);
        emit_u8(0x95);
        emit_u8(0xc0); /* setne al */
        break;
    case TOK_LT:
        emit_u8(0x0f);
        emit_u8(0x9c);
        emit_u8(0xc0); /* setl al */
        break;
    case TOK_GT:
        emit_u8(0x0f);
        emit_u8(0x9f);
        emit_u8(0xc0); /* setg al */
        break;
    case TOK_LEQ:
        emit_u8(0x0f);
        emit_u8(0x9e);
        emit_u8(0xc0); /* setle al */
        break;
    case TOK_GEQ:
        emit_u8(0x0f);
        emit_u8(0x9d);
        emit_u8(0xc0); /* setge al */
        break;
    default:
        break;
    }
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}
