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

static int loop_depth = 0;
static int loop_continue_addrs[MAX_LOOP_DEPTH];
static int loop_break_patches[MAX_LOOP_DEPTH][MAX_LOOP_PATCHES];
static int loop_break_counts[MAX_LOOP_DEPTH];
static int loop_continue_patches[MAX_LOOP_DEPTH][MAX_LOOP_PATCHES];
static int loop_continue_counts[MAX_LOOP_DEPTH];

void init_loop_stack(void) {
    loop_depth = 0;
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
