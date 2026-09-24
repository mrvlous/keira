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
#include "symbols.h"

char global_names[MAX_GLOBALS * 32];
int global_offsets[MAX_GLOBALS];
int global_sizes[MAX_GLOBALS];
int global_count = 0;

int lookup_global(const char *name) {
    int i = 0;
    while (i < global_count) {
        if (k_strcmp(global_names + i * 32, name) == 0) {
            return global_offsets[i];
        }
        i++;
    }
    return -1;
}

int add_global(const char *name, int size) {
    int offset = lookup_global(name);
    if (offset != -1)
        return offset;

    if (global_count >= MAX_GLOBALS) {
        error_msg("Global symbol table overflow");
        return -1;
    }

    /* Align to 8 bytes for quadwords */
    if (size >= 8 && (data_idx % 8) != 0) {
        data_idx += (8 - (data_idx % 8));
    } else if (size >= 4 && (data_idx % 4) != 0) {
        data_idx += (4 - (data_idx % 4));
    }

    int current_offset = data_idx;
    data_idx += size;

    k_strcpy(global_names + global_count * 32, name);
    global_offsets[global_count] = current_offset;
    global_sizes[global_count] = size;
    global_count++;
    return current_offset;
}
