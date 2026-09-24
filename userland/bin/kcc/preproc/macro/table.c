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
#include "preproc.h"

static struct Macro macro_table[MAX_MACROS];
static int macro_count = 0;

void init_preprocessor(void) {
    macro_count = 0;
    k_memset((char *)macro_table, 0, sizeof(macro_table));

    define_macro("__KEIRA__", "1");
    define_macro("__KCC__", "1");
#if defined(__i386__) || defined(__i686__)
    define_macro("__i386__", "1");
#else
    define_macro("__x86_64__", "1");
#endif
    define_macro("NULL", "0");
}

int define_macro(const char *name, const char *val) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            if (val && val[0]) {
                macro_table[i].has_val = 1;
                k_strcpy(macro_table[i].val, val);
            } else {
                macro_table[i].has_val = 0;
                macro_table[i].val[0] = '\0';
            }
            return 0;
        }
    }
    if (macro_count < MAX_MACROS) {
        k_strcpy(macro_table[macro_count].name, name);
        if (val && val[0]) {
            macro_table[macro_count].has_val = 1;
            k_strcpy(macro_table[macro_count].val, val);
        } else {
            macro_table[macro_count].has_val = 0;
            macro_table[macro_count].val[0] = '\0';
        }
        macro_table[macro_count].is_defined = 1;
        macro_count++;
        return 0;
    }
    return -1;
}

int undef_macro(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            macro_table[i].is_defined = 0;
            macro_table[i].has_val = 0;
            return 0;
        }
    }
    return -1;
}

int is_macro_defined(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            return 1;
        }
    }
    return 0;
}

const char *get_macro_val(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            if (macro_table[i].has_val) {
                return macro_table[i].val;
            }
            return "";
        }
    }
    return 0;
}
