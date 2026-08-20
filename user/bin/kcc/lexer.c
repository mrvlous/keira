/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "lexer.h"

#include "common.h"

char *src_ptr = 0;
char token_string[256];
int token_num = 0;
int tok = 0;

void skip_whitespace(void) {
    while (1) {
        if (*src_ptr == ' ' || *src_ptr == '\t' || *src_ptr == '\r' || *src_ptr == '\n') {
            src_ptr = src_ptr + 1;
        } else if (*src_ptr == '#') {
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr = src_ptr + 1;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '/') {
            src_ptr = src_ptr + 2;
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr = src_ptr + 1;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '*') {
            src_ptr = src_ptr + 2;
            while (*src_ptr && !(*src_ptr == '*' && *(src_ptr + 1) == '/')) {
                src_ptr = src_ptr + 1;
            }
            if (*src_ptr) {
                src_ptr = src_ptr + 2;
            }
        } else {
            break;
        }
    }
}

int next_token(void) {
    skip_whitespace();
    if (!*src_ptr) {
        return TOK_EOF;
    }

    if ((*src_ptr >= 'a' && *src_ptr <= 'z') || (*src_ptr >= 'A' && *src_ptr <= 'Z') ||
        *src_ptr == '_') {
        int len = 0;
        while ((*src_ptr >= 'a' && *src_ptr <= 'z') || (*src_ptr >= 'A' && *src_ptr <= 'Z') ||
               (*src_ptr >= '0' && *src_ptr <= '9') || *src_ptr == '_') {
            if (len < 255) {
                token_string[len] = *src_ptr;
                len = len + 1;
            }
            src_ptr = src_ptr + 1;
        }
        token_string[len] = 0;

        if (k_strcmp(token_string, "int") == 0)
            return TOK_INT;
        if (k_strcmp(token_string, "char") == 0)
            return TOK_CHAR;
        if (k_strcmp(token_string, "void") == 0)
            return TOK_VOID;
        if (k_strcmp(token_string, "printf") == 0)
            return TOK_PRINTF;
        if (k_strcmp(token_string, "return") == 0)
            return TOK_RETURN;
        if (k_strcmp(token_string, "if") == 0)
            return TOK_IF;
        if (k_strcmp(token_string, "else") == 0)
            return TOK_ELSE;
        if (k_strcmp(token_string, "while") == 0)
            return TOK_WHILE;
        if (k_strcmp(token_string, "for") == 0)
            return TOK_FOR;
        return TOK_IDENT;
    }

    if (*src_ptr >= '0' && *src_ptr <= '9') {
        token_num = 0;
        while (*src_ptr >= '0' && *src_ptr <= '9') {
            token_num = token_num * 10 + (*src_ptr - '0');
            src_ptr = src_ptr + 1;
        }
        return TOK_NUM;
    }

    if (*src_ptr == '\'') {
        src_ptr = src_ptr + 1;
        int val = 0;
        if (*src_ptr == '\\') {
            src_ptr = src_ptr + 1;
            if (*src_ptr == 'n')
                val = 10;
            else if (*src_ptr == 't')
                val = 9;
            else if (*src_ptr == 'r')
                val = 13;
            else if (*src_ptr == '0')
                val = 0;
            else
                val = *src_ptr;
            src_ptr = src_ptr + 1;
        } else {
            val = *src_ptr;
            src_ptr = src_ptr + 1;
        }
        if (*src_ptr == '\'') {
            src_ptr = src_ptr + 1;
        }
        token_num = val;
        return TOK_NUM;
    }

    if (*src_ptr == '"') {
        src_ptr = src_ptr + 1;
        int len = 0;
        while (*src_ptr && *src_ptr != '"') {
            if (*src_ptr == '\\' && *(src_ptr + 1) == 'n') {
                token_string[len] = '\n';
                len = len + 1;
                src_ptr = src_ptr + 2;
            } else {
                token_string[len] = *src_ptr;
                len = len + 1;
                src_ptr = src_ptr + 1;
            }
        }
        if (*src_ptr == '"') {
            src_ptr = src_ptr + 1;
        }
        token_string[len] = 0;
        return TOK_STRING;
    }

    char c = *src_ptr;
    src_ptr = src_ptr + 1;
    if (c == '(')
        return TOK_LPAREN;
    if (c == ')')
        return TOK_RPAREN;
    if (c == '{')
        return TOK_LBRACE;
    if (c == '}')
        return TOK_RBRACE;
    if (c == ';')
        return TOK_SEMICOLON;
    if (c == ',')
        return TOK_COMMA;
    if (c == '[')
        return TOK_LBRACKET;
    if (c == ']')
        return TOK_RBRACKET;
    if (c == '+')
        return TOK_PLUS;
    if (c == '-')
        return TOK_MINUS;
    if (c == '*')
        return TOK_STAR;
    if (c == '/')
        return TOK_SLASH;
    if (c == '<') {
        if (*src_ptr == '=') {
            src_ptr = src_ptr + 1;
            return TOK_LEQ;
        }
        return TOK_LT;
    }
    if (c == '>') {
        if (*src_ptr == '=') {
            src_ptr = src_ptr + 1;
            return TOK_GEQ;
        }
        return TOK_GT;
    }
    if (c == '=') {
        if (*src_ptr == '=') {
            src_ptr = src_ptr + 1;
            return TOK_EQ;
        }
        return TOK_ASSIGN;
    }
    if (c == '!') {
        if (*src_ptr == '=') {
            src_ptr = src_ptr + 1;
            return TOK_NEQ;
        }
    }
    if (c == '&') {
        if (*src_ptr == '&') {
            src_ptr = src_ptr + 1;
            return TOK_AND;
        }
    }
    if (c == '|') {
        if (*src_ptr == '|') {
            src_ptr = src_ptr + 1;
            return TOK_OR;
        }
    }

    return TOK_EOF;
}
