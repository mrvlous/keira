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

/* Token Categories */
enum TokenType {
    TOK_EOF = 0,
    TOK_IDENT,
    TOK_NUM,
    TOK_STRING,

    /* Types & Storage */
    TOK_INT,
    TOK_CHAR,
    TOK_VOID,
    TOK_SHORT,
    TOK_LONG,
    TOK_UNSIGNED,
    TOK_SIGNED,

    /* Control Flow Keywords */
    TOK_RETURN,
    TOK_IF,
    TOK_ELSE,
    TOK_WHILE,
    TOK_FOR,
    TOK_DO,
    TOK_BREAK,
    TOK_CONTINUE,
    TOK_SIZEOF,

    /* Built-in Utility Keywords */
    TOK_PRINTF,
    TOK_SYSCALL,

    /* Delimiters & Grouping */
    TOK_LPAREN,    /* ( */
    TOK_RPAREN,    /* ) */
    TOK_LBRACE,    /* { */
    TOK_RBRACE,    /* } */
    TOK_LBRACKET,  /* [ */
    TOK_RBRACKET,  /* ] */
    TOK_SEMICOLON, /* ; */
    TOK_COMMA,     /* , */
    TOK_COLON,     /* : */
    TOK_QUESTION,  /* ? */

    /* Arithmetic Operators */
    TOK_PLUS,  /* + */
    TOK_MINUS, /* - */
    TOK_STAR,  /* * */
    TOK_SLASH, /* / */
    TOK_MOD,   /* % */

    /* Bitwise Operators */
    TOK_AMP,   /* & */
    TOK_PIPE,  /* | */
    TOK_CARET, /* ^ */
    TOK_TILDE, /* ~ */
    TOK_SHL,   /* << */
    TOK_SHR,   /* >> */

    /* Logical Operators */
    TOK_AND, /* && */
    TOK_OR,  /* || */
    TOK_NOT, /* ! */

    /* Relational & Equality */
    TOK_LT,  /* < */
    TOK_GT,  /* > */
    TOK_LEQ, /* <= */
    TOK_GEQ, /* >= */
    TOK_EQ,  /* == */
    TOK_NEQ, /* != */

    /* Increment / Decrement */
    TOK_INC, /* ++ */
    TOK_DEC, /* -- */

    /* Assignments */
    TOK_ASSIGN,     /* = */
    TOK_ADD_ASSIGN, /* += */
    TOK_SUB_ASSIGN, /* -= */
    TOK_MUL_ASSIGN, /* *= */
    TOK_DIV_ASSIGN, /* /= */
    TOK_MOD_ASSIGN, /* %= */
    TOK_AND_ASSIGN, /* &= */
    TOK_OR_ASSIGN,  /* |= */
    TOK_XOR_ASSIGN, /* ^= */
    TOK_SHL_ASSIGN, /* <<= */
    TOK_SHR_ASSIGN  /* >>= */
};

extern char *src_ptr;
extern int line_num;
extern char token_string[256];
extern long token_num;
extern int tok;

void init_lexer(char *src);
void skip_whitespace(void);
int next_token(void);
const char *token_name(int token);

#endif /* _KCC_LEXER_H */
