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

char local_names[MAX_LOCALS * 32];
int local_offsets[MAX_LOCALS];
int local_sizes[MAX_LOCALS];
int local_count = 0;
int current_local_offset = 0;

int lookup_local(const char *name) {
    int i = 0;
    while (i < local_count) {
        if (k_strcmp(local_names + i * 32, name) == 0) {
            return local_offsets[i];
        }
        i++;
    }
    return 0;
}

int add_local(const char *name, int size) {
    int offset = lookup_local(name);
    if (offset != 0)
        return offset;

    if (local_count >= MAX_LOCALS) {
        error_msg("Local symbol table overflow");
        return 0;
    }

    int slot_size = size;
    if (slot_size < 8)
        slot_size = 8;
    else if ((slot_size % 8) != 0)
        slot_size += (8 - (slot_size % 8));

    current_local_offset += slot_size;
    int var_offset = -current_local_offset;

    k_strcpy(local_names + local_count * 32, name);
    local_offsets[local_count] = var_offset;
    local_sizes[local_count] = size;
    local_count++;
    return var_offset;
}

void clear_locals(void) {
    local_count = 0;
    current_local_offset = 0;
}
