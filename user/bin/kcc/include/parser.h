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

/* Expression Parsing Hierarchy */
void primary_expr(void);
void postfix_expr(void);
void unary_expr(void);
void mul_expr(void);
void add_expr(void);
void shift_expr(void);
void rel_expr(void);
void eq_expr(void);
void bit_and_expr(void);
void bit_xor_expr(void);
void bit_or_expr(void);
void log_and_expr(void);
void log_or_expr(void);
void assignment_expr(void);
void expression(void);

/* Statements & Program Structures */
void statement(void);
void block(void);
void compile_global_declarations(void);

#endif /* _KCC_PARSER_H */
