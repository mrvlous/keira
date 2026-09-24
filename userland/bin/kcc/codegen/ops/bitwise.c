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

/* Bitwise Operations (rcx = LHS, rax = RHS) */
void emit_bit_and(void) {
    /* and rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x21);
    emit_u8(0xc8);
}

void emit_bit_or(void) {
    /* or rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x09);
    emit_u8(0xc8);
}

void emit_bit_xor(void) {
    /* xor rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x31);
    emit_u8(0xc8);
}

void emit_bit_not(void) {
    /* not rax/eax */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xd0);
}

void emit_shl(void) {
    /* xchg rax/eax, rcx/ecx; shl rax/eax, cl */
    REX_W();
    emit_u8(0x91); /* xchg */
    REX_W();
    emit_u8(0xd3);
    emit_u8(0xe0); /* shl */
}

void emit_shr(void) {
    /* xchg rax/eax, rcx/ecx; sar rax/eax, cl */
    REX_W();
    emit_u8(0x91); /* xchg */
    REX_W();
    emit_u8(0xd3);
    emit_u8(0xf8); /* sar */
}
