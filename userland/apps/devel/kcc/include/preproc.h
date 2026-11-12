// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#ifndef _KCC_PREPROC_H
#define _KCC_PREPROC_H

#define MAX_MACROS 256
#define MAX_MACRO_NAME 64
#define MAX_MACRO_VAL 128
#define MAX_IF_DEPTH 16
#define MAX_INCLUDE_DEPTH 8

struct Macro {
    char name[MAX_MACRO_NAME];
    char val[MAX_MACRO_VAL];
    int has_val;
    int is_defined;
};

void init_preprocessor(void);
int define_macro(const char *name, const char *val);
int undef_macro(const char *name);
int is_macro_defined(const char *name);
const char *get_macro_val(const char *name);

int preprocess_source(const char *input, char *output, int max_out, const char *cur_dir);

#endif /* _KCC_PREPROC_H */
