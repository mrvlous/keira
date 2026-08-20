/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KCC_LEXER_H
#define _KCC_LEXER_H

#define TOK_EOF 0
#define TOK_INT 1
#define TOK_VOID 2
#define TOK_MAIN 3
#define TOK_PRINTF 4
#define TOK_RETURN 5
#define TOK_IDENT 6
#define TOK_NUM 7
#define TOK_STRING 8
#define TOK_LPAREN 9
#define TOK_RPAREN 10
#define TOK_LBRACE 11
#define TOK_RBRACE 12
#define TOK_SEMICOLON 13
#define TOK_IF 14
#define TOK_ELSE 15
#define TOK_WHILE 16
#define TOK_ASSIGN 17
#define TOK_PLUS 18
#define TOK_MINUS 19
#define TOK_STAR 20
#define TOK_SLASH 21
#define TOK_LT 22
#define TOK_GT 23
#define TOK_EQ 24
#define TOK_NEQ 25
#define TOK_COMMA 26
#define TOK_LBRACKET 27
#define TOK_RBRACKET 28
#define TOK_CHAR 29
#define TOK_FOR 30
#define TOK_LEQ 31
#define TOK_GEQ 32
#define TOK_AND 33
#define TOK_OR 34

extern char *src_ptr;
extern char token_string[256];
extern int token_num;
extern int tok;

void skip_whitespace(void);
int next_token(void);

#endif /* _KCC_LEXER_H */
