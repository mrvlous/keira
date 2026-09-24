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

/* Pointer Dereferencing */
void emit_deref(int size) {
    if (size == 1) {
        /* movzx rax/eax, byte ptr [rax/eax] */
        REX_W();
        emit_u8(0x0f);
        emit_u8(0xb6);
        emit_u8(0x00);
    } else {
        /* mov rax/eax, [rax/eax] */
        REX_W();
        emit_u8(0x8b);
        emit_u8(0x00);
    }
}

void emit_store_deref(int size) {
    /* rdx/edx has pointer address, rax/eax has value to write */
    if (size == 1) {
        /* mov byte ptr [rdx/edx], al */
        emit_u8(0x88);
        emit_u8(0x02);
    } else {
        /* mov [rdx/edx], rax/eax */
        REX_W();
        emit_u8(0x89);
        emit_u8(0x02);
    }
}
