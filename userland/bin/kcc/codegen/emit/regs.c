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

void emit_load_imm(long val) {
#if defined(__i386__) || defined(__i686__)
    /* mov eax, imm32 */
    emit_u8(0xb8);
    emit_u32((unsigned int)val);
#else
    /* mov rax, imm64 */
    emit_u8(0x48);
    emit_u8(0xb8);
    emit_u64((unsigned long)val);
#endif
}

void emit_push_rax(void) {
    emit_u8(0x50); /* push rax / eax */
}

void emit_pop_rax(void) {
    emit_u8(0x58); /* pop rax / eax */
}

void emit_pop_rcx(void) {
    emit_u8(0x59); /* pop rcx / ecx */
}

void emit_pop_rdx(void) {
    emit_u8(0x5a); /* pop rdx / edx */
}

void emit_pop_rsi(void) {
    emit_u8(0x5e); /* pop rsi / esi */
}

void emit_pop_rdi(void) {
    emit_u8(0x5f); /* pop rdi / edi */
}

void emit_pop_r8(void) {
    emit_u8(0x41);
    emit_u8(0x58); /* pop r8 */
}

void emit_pop_r9(void) {
    emit_u8(0x41);
    emit_u8(0x59); /* pop r9 */
}
