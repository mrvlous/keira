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

void emit_u8(unsigned char val) {
    if (code_idx >= MAX_CODE_SIZE) {
        error_msg("Code segment overflow");
        return;
    }
    code_buf[code_idx++] = val;
}

void emit_u16(unsigned short val) {
    emit_u8((unsigned char)(val & 0xFF));
    emit_u8((unsigned char)((val >> 8) & 0xFF));
}

void emit_u32(unsigned int val) {
    emit_u8((unsigned char)(val & 0xFF));
    emit_u8((unsigned char)((val >> 8) & 0xFF));
    emit_u8((unsigned char)((val >> 16) & 0xFF));
    emit_u8((unsigned char)((val >> 24) & 0xFF));
}

void emit_u64(uint64_t val) {
    emit_u32((unsigned int)(val & 0xFFFFFFFF));
    emit_u32((unsigned int)((val >> 32) & 0xFFFFFFFF));
}
