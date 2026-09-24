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

char function_names[MAX_FUNCTIONS * 32];
int function_addresses[MAX_FUNCTIONS];
int function_count = 0;

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
