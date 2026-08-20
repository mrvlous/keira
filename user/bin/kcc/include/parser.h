/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KCC_PARSER_H
#define _KCC_PARSER_H

void match(int expected);
void primary_expr(void);
void mul_expr(void);
void add_expr(void);
void expression(void);
void statement(void);
void block(void);
void compile_global_declarations(void);

#endif /* _KCC_PARSER_H */
