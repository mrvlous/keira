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
#include "symbols.h"

/* Global Variables & Data Segment Addressing */
void emit_addr_global(int offset) {
#if defined(__i386__) || defined(__i686__)
    emit_u8(0xb8);
    int patch_pos = code_idx;
    emit_u32((unsigned int)offset);
#else
    emit_u8(0x48);
    emit_u8(0xb8);
    int patch_pos = code_idx;
    emit_u64((unsigned long)offset);
#endif

    if (val_patch_count < MAX_VAL_PATCHES) {
        val_patch_addresses[val_patch_count] = patch_pos;
        val_patch_offsets[val_patch_count] = offset;
        val_patch_count++;
    }
}

void emit_load_global(int offset, int size) {
    emit_addr_global(offset);
    emit_deref(size);
}

void emit_store_global(int offset, int size) {
    /* rax/eax has value to store */
    emit_push_rax();
    emit_addr_global(offset);
    /* rax/eax = address, top of stack = value */
    REX_W();
    emit_u8(0x89);
    emit_u8(0xc2);  /* mov rdx/edx, rax/eax (address) */
    emit_pop_rax(); /* rax/eax = value */
    emit_store_deref(size);
}

void emit_inc_global(int offset, int is_post, int is_dec) {
    emit_load_global(offset, 8);
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
    emit_store_global(offset, 8);
    if (is_post) {
        emit_pop_rax(); /* return original */
    }
}
