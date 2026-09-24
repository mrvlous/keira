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

/* Logical & Unary Operations */
void emit_log_and(void) {
    /* test rcx, rcx; setne cl; test rax, rax; setne al; and al, cl; movzx rax, al */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc9); /* test rcx/ecx, rcx/ecx */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc1); /* setne cl */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0); /* test rax/eax, rax/eax */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc0); /* setne al */
    emit_u8(0x20);
    emit_u8(0xc8); /* and al, cl */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}

void emit_log_or(void) {
    /* or rcx, rax; test rcx, rcx; setne al; movzx rax, al */
    REX_W();
    emit_u8(0x09);
    emit_u8(0xc1); /* or rcx/ecx, rax/eax */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc9); /* test rcx/ecx, rcx/ecx */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc0); /* setne al */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}

void emit_log_not(void) {
    /* test rax, rax; sete al; movzx rax, al */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x94);
    emit_u8(0xc0);
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0);
}

void emit_neg(void) {
    /* neg rax/eax */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xd8);
}
