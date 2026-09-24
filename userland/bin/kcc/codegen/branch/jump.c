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

/* Control Flow, Branches & Jump Patching */
int emit_jmp_forward(void) {
    emit_u8(0xe9);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

void emit_jmp_backward(int target_addr) {
    emit_u8(0xe9);
    int rel = target_addr - (code_idx + 4);
    emit_u32((unsigned int)rel);
}

int emit_jz_forward(void) {
    /* test rax/eax, rax/eax; jz rel32 */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x84);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

int emit_jnz_forward(void) {
    /* test rax/eax, rax/eax; jnz rel32 */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x85);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

void patch_jump(int patch_pos, int target_pos) {
    int rel_offset = target_pos - (patch_pos + 4);
    k_memcpy((char *)(code_buf + patch_pos), (char *)&rel_offset, 4);
}
