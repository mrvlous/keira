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

#define MAX_GLOBALS 256
#define MAX_LOCALS 128
#define MAX_FUNCTIONS 128
#define MAX_PATCHES 512
#define MAX_VAL_PATCHES 1024
#define MAX_LOOP_DEPTH 32
#define MAX_LOOP_PATCHES 64

extern char global_names[MAX_GLOBALS * 32];
extern int global_offsets[MAX_GLOBALS];
extern int global_sizes[MAX_GLOBALS];
extern int global_count;

extern char local_names[MAX_LOCALS * 32];
extern int local_offsets[MAX_LOCALS];
extern int local_sizes[MAX_LOCALS];
extern int local_count;

extern char function_names[MAX_FUNCTIONS * 32];
extern int function_addresses[MAX_FUNCTIONS];
extern int function_count;

extern char patch_names[MAX_PATCHES * 32];
extern int patch_addresses[MAX_PATCHES];
extern int patch_count;

extern int val_patch_addresses[MAX_VAL_PATCHES];
extern int val_patch_offsets[MAX_VAL_PATCHES];
extern int val_patch_count;

void init_symbols(void);

int lookup_global(const char *name);
int add_global(const char *name, int size);

int lookup_local(const char *name);
int add_local(const char *name, int size);
void clear_locals(void);

int lookup_function(const char *name);
void add_function(const char *name, int address);

/* Loop break / continue control flow management */
void push_loop(int continue_addr);
void add_loop_break_patch(int patch_addr);
void add_loop_continue_patch(int patch_addr);
void pop_loop(int break_target_addr);

#endif /* _KCC_SYMBOLS_H */
