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

char global_names[MAX_GLOBALS * 32];
int global_offsets[MAX_GLOBALS];
int global_sizes[MAX_GLOBALS];
int global_count = 0;

char local_names[MAX_LOCALS * 32];
int local_offsets[MAX_LOCALS];
int local_sizes[MAX_LOCALS];
int local_count = 0;
int current_local_offset = 0;

char function_names[MAX_FUNCTIONS * 32];
int function_addresses[MAX_FUNCTIONS];
int function_count = 0;

char patch_names[MAX_PATCHES * 32];
int patch_addresses[MAX_PATCHES];
int patch_count = 0;

int val_patch_addresses[MAX_VAL_PATCHES];
int val_patch_offsets[MAX_VAL_PATCHES];
int val_patch_count = 0;

/* Loop Control Stack */
static int loop_depth = 0;
static int loop_continue_addrs[MAX_LOOP_DEPTH];
static int loop_break_patches[MAX_LOOP_DEPTH][MAX_LOOP_PATCHES];
static int loop_break_counts[MAX_LOOP_DEPTH];
static int loop_continue_patches[MAX_LOOP_DEPTH][MAX_LOOP_PATCHES];
static int loop_continue_counts[MAX_LOOP_DEPTH];

void init_symbols(void) {
    global_count = 0;
    local_count = 0;
    current_local_offset = 0;
    function_count = 0;
    patch_count = 0;
    val_patch_count = 0;
    loop_depth = 0;
}

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

    /* Variable allocation on stack (each local aligned to at least 8 bytes for 64-bit cleanliness)
     */
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

int lookup_function(const char *name) {
    int i = 0;
    while (i < function_count) {
        if (k_strcmp(function_names + i * 32, name) == 0) {
            return function_addresses[i];
        }
        i++;
    }
    return -1;
}

void add_function(const char *name, int address) {
    if (function_count >= MAX_FUNCTIONS) {
        error_msg("Function symbol table overflow");
        return;
    }
    k_strcpy(function_names + function_count * 32, name);
    function_addresses[function_count] = address;
    function_count++;
}

void push_loop(int continue_addr) {
    if (loop_depth >= MAX_LOOP_DEPTH) {
        error_msg("Loop nesting depth exceeded");
        return;
    }
    loop_continue_addrs[loop_depth] = continue_addr;
    loop_break_counts[loop_depth] = 0;
    loop_continue_counts[loop_depth] = 0;
    loop_depth++;
}

void add_loop_break_patch(int patch_addr) {
    if (loop_depth <= 0) {
        error_msg("'break' statement not within a loop");
        return;
    }
    int cur = loop_depth - 1;
    if (loop_break_counts[cur] < MAX_LOOP_PATCHES) {
        loop_break_patches[cur][loop_break_counts[cur]++] = patch_addr;
    }
}

void add_loop_continue_patch(int patch_addr) {
    if (loop_depth <= 0) {
        error_msg("'continue' statement not within a loop");
        return;
    }
    int cur = loop_depth - 1;
    if (loop_continue_counts[cur] < MAX_LOOP_PATCHES) {
        loop_continue_patches[cur][loop_continue_counts[cur]++] = patch_addr;
    }
}

void pop_loop(int break_target_addr) {
    if (loop_depth <= 0)
        return;
    loop_depth--;
    int cur = loop_depth;

    /* Patch all break jumps to break_target_addr */
    int i;
    for (i = 0; i < loop_break_counts[cur]; i++) {
        int patch_pos = loop_break_patches[cur][i];
        int rel_offset = break_target_addr - (patch_pos + 4);
        k_memcpy((char *)(code_buf + patch_pos), (char *)&rel_offset, 4);
    }

    /* Patch any deferred continue jumps if continue_addr was forward */
    if (loop_continue_addrs[cur] != -1) {
        int cont_addr = loop_continue_addrs[cur];
        for (i = 0; i < loop_continue_counts[cur]; i++) {
            int patch_pos = loop_continue_patches[cur][i];
            int rel_offset = cont_addr - (patch_pos + 4);
            k_memcpy((char *)(code_buf + patch_pos), (char *)&rel_offset, 4);
        }
    }
}
