/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KCC_SYMBOLS_H
#define _KCC_SYMBOLS_H

extern char global_names[64 * 32];
extern int global_offsets[64];
extern int global_count;

extern char local_names[32 * 32];
extern int local_offsets[32];
extern int local_count;

extern char function_names[32 * 32];
extern int function_addresses[32];
extern int function_count;

extern char patch_names[128 * 32];
extern int patch_addresses[128];
extern int patch_count;

extern int val_patch_addresses[512];
extern int val_patch_offsets[512];
extern int val_patch_count;

int lookup_global(const char *name);
int add_global(const char *name, int size);

int lookup_local(const char *name);
int add_local(const char *name);

int lookup_function(const char *name);
void add_function(const char *name, int address);

#endif /* _KCC_SYMBOLS_H */
