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

/* Local Variables & Stack Frame Handling */
void emit_load_local(int offset, int size) {
    if (size == 1) {
        /* movzx rax/eax, byte ptr [rbp/ebp + offset] */
        REX_W();
        emit_u8(0x0f);
        emit_u8(0xb6);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    } else {
        /* mov rax/eax, [rbp/ebp + offset] */
        REX_W();
        emit_u8(0x8b);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    }
}

void emit_store_local(int offset, int size) {
    if (size == 1) {
        /* mov byte ptr [rbp/ebp + offset], al */
        emit_u8(0x88);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    } else {
        /* mov [rbp/ebp + offset], rax/eax */
        REX_W();
        emit_u8(0x89);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    }
}

void emit_addr_local(int offset) {
    /* lea rax/eax, [rbp/ebp + offset] */
    REX_W();
    emit_u8(0x8d);
    if (offset >= -128 && offset <= 127) {
        emit_u8(0x45);
        emit_u8((unsigned char)offset);
    } else {
        emit_u8(0x85);
        emit_u32((unsigned int)offset);
    }
}

void emit_inc_local(int offset, int is_post, int is_dec) {
    emit_load_local(offset, 8);
    if (is_post) {
        emit_push_rax(); /* save original value for expression result */
    }
    if (is_dec) {
        REX_W();
        emit_u8(0xff);
        emit_u8(0xc8); /* dec rax/eax */
    } else {
        REX_W();
        emit_u8(0xff);
        emit_u8(0xc0); /* inc rax/eax */
    }
    emit_store_local(offset, 8);
    if (is_post) {
        emit_pop_rax(); /* return original */
    }
}
