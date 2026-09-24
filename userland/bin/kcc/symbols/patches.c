/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "symbols.h"

char patch_names[MAX_PATCHES * 32];
int patch_addresses[MAX_PATCHES];
int patch_count = 0;

int val_patch_addresses[MAX_VAL_PATCHES];
int val_patch_offsets[MAX_VAL_PATCHES];
int val_patch_count = 0;

void init_symbols(void) {
    global_count = 0;
    local_count = 0;
    current_local_offset = 0;
    function_count = 0;
    patch_count = 0;
    val_patch_count = 0;
    init_loop_stack();
}
