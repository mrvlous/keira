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

/* Binary Arithmetic (rcx = LHS, rax = RHS) */
void emit_add(void) {
    /* add rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x01);
    emit_u8(0xc8);
}

void emit_sub(void) {
    /* sub rcx/ecx, rax/eax; mov rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x29);
    emit_u8(0xc1);
    REX_W();
    emit_u8(0x89);
    emit_u8(0xc8);
}

void emit_imul(void) {
    /* imul rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xaf);
    emit_u8(0xc1);
}

void emit_idiv(void) {
    /* xchg rax, rcx; cqo/cdq; idiv rcx */
    REX_W();
    emit_u8(0x91); /* xchg rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x99); /* cqo / cdq */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xf9); /* idiv rcx/ecx */
}

void emit_imod(void) {
    /* xchg rax, rcx; cqo/cdq; idiv rcx; mov rax, rdx */
    REX_W();
    emit_u8(0x91); /* xchg rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x99); /* cqo / cdq */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xf9); /* idiv rcx/ecx */
    REX_W();
    emit_u8(0x89);
    emit_u8(0xd0); /* mov rax/eax, rdx/edx */
}
