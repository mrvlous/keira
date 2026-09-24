/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "common.h"

void write_u8(char *buf, int offset, int val) {
    buf[offset] = (char)val;
}

void write_u16(char *buf, int offset, int val) {
    buf[offset] = (char)(val & 255);
    buf[offset + 1] = (char)((val >> 8) & 255);
}

void write_u32(char *buf, int offset, int val) {
    buf[offset] = (char)(val & 255);
    buf[offset + 1] = (char)((val >> 8) & 255);
    buf[offset + 2] = (char)((val >> 16) & 255);
    buf[offset + 3] = (char)((val >> 24) & 255);
}

void write_u64(char *buf, int offset, uint64_t val) {
    buf[offset] = (char)(val & 255);
    buf[offset + 1] = (char)((val >> 8) & 255);
    buf[offset + 2] = (char)((val >> 16) & 255);
    buf[offset + 3] = (char)((val >> 24) & 255);
    buf[offset + 4] = (char)((val >> 32) & 255);
    buf[offset + 5] = (char)((val >> 40) & 255);
    buf[offset + 6] = (char)((val >> 48) & 255);
    buf[offset + 7] = (char)((val >> 56) & 255);
}
