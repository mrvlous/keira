/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "symbols.h"

#include "common.h"

char global_names[64 * 32];
int global_offsets[64];
int global_count = 0;

char local_names[32 * 32];
int local_offsets[32];
int local_count = 0;

char function_names[32 * 32];
int function_addresses[32];
int function_count = 0;

char patch_names[128 * 32];
int patch_addresses[128];
int patch_count = 0;

int val_patch_addresses[512];
int val_patch_offsets[512];
int val_patch_count = 0;

int lookup_global(const char *name) {
    int i = 0;
    while (i < global_count) {
        if (k_strcmp(global_names + i * 32, name) == 0) {
            return global_offsets[i];
        }
        i = i + 1;
    }
    return 0 - 1;
}

int add_global(const char *name, int size) {
    int offset = lookup_global(name);
    if (offset != 0 - 1)
        return offset;

    int current_offset = data_idx;
    data_idx = data_idx + size;

    k_strcpy(global_names + global_count * 32, name);
    global_offsets[global_count] = current_offset;
    global_count = global_count + 1;
    return current_offset;
}

int lookup_local(const char *name) {
    int i = 0;
    while (i < local_count) {
        if (k_strcmp(local_names + i * 32, name) == 0) {
            return local_offsets[i];
        }
        i = i + 1;
    }
    return 0;
}

int add_local(const char *name) {
    int offset = lookup_local(name);
    if (offset != 0)
        return offset;

    local_count = local_count + 1;
    k_strcpy(local_names + (local_count - 1) * 32, name);
    local_offsets[local_count - 1] = 0 - (8 * local_count);
    return 0 - (8 * local_count);
}

int lookup_function(const char *name) {
    int i = 0;
    while (i < function_count) {
        if (k_strcmp(function_names + i * 32, name) == 0) {
            return function_addresses[i];
        }
        i = i + 1;
    }
    return 0 - 1;
}

void add_function(const char *name, int address) {
    k_strcpy(function_names + function_count * 32, name);
    function_addresses[function_count] = address;
    function_count = function_count + 1;
}
