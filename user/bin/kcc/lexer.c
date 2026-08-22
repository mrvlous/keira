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
int line_num = 1;
char token_string[256];
long token_num = 0;
int tok = 0;

void init_lexer(char *src) {
    src_ptr = src;
    line_num = 1;
    token_num = 0;
    token_string[0] = '\0';
    tok = 0;
}

void skip_whitespace(void) {
    while (*src_ptr != '\0') {
        if (*src_ptr == ' ' || *src_ptr == '\t' || *src_ptr == '\r') {
            src_ptr++;
        } else if (*src_ptr == '\n') {
            line_num++;
            src_ptr++;
        } else if (*src_ptr == '#') {
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr++;
            }
            if (*src_ptr == '\n') {
                line_num++;
                src_ptr++;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '/') {
            src_ptr += 2;
            while (*src_ptr && *src_ptr != '\n') {
                src_ptr++;
            }
            if (*src_ptr == '\n') {
                line_num++;
                src_ptr++;
            }
        } else if (*src_ptr == '/' && *(src_ptr + 1) == '*') {
            src_ptr += 2;
            while (*src_ptr && !(*src_ptr == '*' && *(src_ptr + 1) == '/')) {
                if (*src_ptr == '\n')
                    line_num++;
                src_ptr++;
            }
            if (*src_ptr) {
                src_ptr += 2;
            }
        } else {
            break;
        }
    }
}

int next_token(void) {
    skip_whitespace();
    if (!*src_ptr) {
        tok = TOK_EOF;
        return tok;
    }

    /* Identifiers and Keywords */
    if ((*src_ptr >= 'a' && *src_ptr <= 'z') || (*src_ptr >= 'A' && *src_ptr <= 'Z') ||
        *src_ptr == '_') {
        int len = 0;
        while ((*src_ptr >= 'a' && *src_ptr <= 'z') || (*src_ptr >= 'A' && *src_ptr <= 'Z') ||
               (*src_ptr >= '0' && *src_ptr <= '9') || *src_ptr == '_') {
            if (len < 255) {
                token_string[len++] = *src_ptr;
            }
            src_ptr++;
        }
        token_string[len] = '\0';

        if (k_strcmp(token_string, "int") == 0)
            tok = TOK_INT;
        else if (k_strcmp(token_string, "char") == 0)
            tok = TOK_CHAR;
        else if (k_strcmp(token_string, "void") == 0)
            tok = TOK_VOID;
        else if (k_strcmp(token_string, "short") == 0)
            tok = TOK_SHORT;
        else if (k_strcmp(token_string, "long") == 0)
            tok = TOK_LONG;
        else if (k_strcmp(token_string, "unsigned") == 0)
            tok = TOK_UNSIGNED;
        else if (k_strcmp(token_string, "signed") == 0)
            tok = TOK_SIGNED;
        else if (k_strcmp(token_string, "return") == 0)
            tok = TOK_RETURN;
        else if (k_strcmp(token_string, "if") == 0)
            tok = TOK_IF;
        else if (k_strcmp(token_string, "else") == 0)
            tok = TOK_ELSE;
        else if (k_strcmp(token_string, "while") == 0)
            tok = TOK_WHILE;
        else if (k_strcmp(token_string, "for") == 0)
            tok = TOK_FOR;
        else if (k_strcmp(token_string, "do") == 0)
            tok = TOK_DO;
        else if (k_strcmp(token_string, "break") == 0)
            tok = TOK_BREAK;
        else if (k_strcmp(token_string, "continue") == 0)
            tok = TOK_CONTINUE;
        else if (k_strcmp(token_string, "sizeof") == 0)
            tok = TOK_SIZEOF;
        else if (k_strcmp(token_string, "printf") == 0)
            tok = TOK_PRINTF;
        else if (k_strcmp(token_string, "syscall") == 0)
            tok = TOK_SYSCALL;
        else
            tok = TOK_IDENT;

        return tok;
    }

    /* Numeric Literals (Hex or Decimal) */
    if (*src_ptr >= '0' && *src_ptr <= '9') {
        token_num = 0;
        if (*src_ptr == '0' && (*(src_ptr + 1) == 'x' || *(src_ptr + 1) == 'X')) {
            src_ptr += 2;
            while ((*src_ptr >= '0' && *src_ptr <= '9') || (*src_ptr >= 'a' && *src_ptr <= 'f') ||
                   (*src_ptr >= 'A' && *src_ptr <= 'F')) {
                token_num *= 16;
                if (*src_ptr >= '0' && *src_ptr <= '9')
                    token_num += (*src_ptr - '0');
                else if (*src_ptr >= 'a' && *src_ptr <= 'f')
                    token_num += (10 + *src_ptr - 'a');
                else
                    token_num += (10 + *src_ptr - 'A');
                src_ptr++;
            }
        } else {
            while (*src_ptr >= '0' && *src_ptr <= '9') {
                token_num = token_num * 10 + (*src_ptr - '0');
                src_ptr++;
            }
        }
        tok = TOK_NUM;
        return tok;
    }

    /* Character Literals */
    if (*src_ptr == '\'') {
        src_ptr++;
        int val = 0;
        if (*src_ptr == '\\') {
            src_ptr++;
            if (*src_ptr == 'n')
                val = 10;
            else if (*src_ptr == 't')
                val = 9;
            else if (*src_ptr == 'r')
                val = 13;
            else if (*src_ptr == '0')
                val = 0;
            else if (*src_ptr == '\\')
                val = '\\';
            else if (*src_ptr == '\'')
                val = '\'';
            else
                val = *src_ptr;
            src_ptr++;
        } else {
            val = *src_ptr;
            src_ptr++;
        }
        if (*src_ptr == '\'') {
            src_ptr++;
        }
        token_num = val;
        tok = TOK_NUM;
        return tok;
    }

    /* String Literals */
    if (*src_ptr == '"') {
        src_ptr++;
        int len = 0;
        while (*src_ptr && *src_ptr != '"') {
            if (*src_ptr == '\\') {
                src_ptr++;
                if (*src_ptr == 'n')
                    token_string[len++] = '\n';
                else if (*src_ptr == 't')
                    token_string[len++] = '\t';
                else if (*src_ptr == 'r')
                    token_string[len++] = '\r';
                else if (*src_ptr == '0')
                    token_string[len++] = '\0';
                else if (*src_ptr == '\\')
                    token_string[len++] = '\\';
                else if (*src_ptr == '"')
                    token_string[len++] = '"';
                else
                    token_string[len++] = *src_ptr;
                src_ptr++;
            } else {
                token_string[len++] = *src_ptr++;
            }
        }
        if (*src_ptr == '"') {
            src_ptr++;
        }
        token_string[len] = '\0';
        tok = TOK_STRING;
        return tok;
    }

    char c = *src_ptr++;

    /* Single / Multi-char Operators */
    switch (c) {
    case '(':
        tok = TOK_LPAREN;
        return tok;
    case ')':
        tok = TOK_RPAREN;
        return tok;
    case '{':
        tok = TOK_LBRACE;
        return tok;
    case '}':
        tok = TOK_RBRACE;
        return tok;
    case '[':
        tok = TOK_LBRACKET;
        return tok;
    case ']':
        tok = TOK_RBRACKET;
        return tok;
    case ';':
        tok = TOK_SEMICOLON;
        return tok;
    case ',':
        tok = TOK_COMMA;
        return tok;
    case ':':
        tok = TOK_COLON;
        return tok;
    case '?':
        tok = TOK_QUESTION;
        return tok;
    case '~':
        tok = TOK_TILDE;
        return tok;

    case '+':
        if (*src_ptr == '+') {
            src_ptr++;
            tok = TOK_INC;
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_ADD_ASSIGN;
        } else {
            tok = TOK_PLUS;
        }
        return tok;

    case '-':
        if (*src_ptr == '-') {
            src_ptr++;
            tok = TOK_DEC;
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_SUB_ASSIGN;
        } else {
            tok = TOK_MINUS;
        }
        return tok;

    case '*':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_MUL_ASSIGN;
        } else {
            tok = TOK_STAR;
        }
        return tok;

    case '/':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_DIV_ASSIGN;
        } else {
            tok = TOK_SLASH;
        }
        return tok;

    case '%':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_MOD_ASSIGN;
        } else {
            tok = TOK_MOD;
        }
        return tok;

    case '&':
        if (*src_ptr == '&') {
            src_ptr++;
            tok = TOK_AND;
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_AND_ASSIGN;
        } else {
            tok = TOK_AMP;
        }
        return tok;

    case '|':
        if (*src_ptr == '|') {
            src_ptr++;
            tok = TOK_OR;
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_OR_ASSIGN;
        } else {
            tok = TOK_PIPE;
        }
        return tok;

    case '^':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_XOR_ASSIGN;
        } else {
            tok = TOK_CARET;
        }
        return tok;

    case '<':
        if (*src_ptr == '<') {
            src_ptr++;
            if (*src_ptr == '=') {
                src_ptr++;
                tok = TOK_SHL_ASSIGN;
            } else {
                tok = TOK_SHL;
            }
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_LEQ;
        } else {
            tok = TOK_LT;
        }
        return tok;

    case '>':
        if (*src_ptr == '>') {
            src_ptr++;
            if (*src_ptr == '=') {
                src_ptr++;
                tok = TOK_SHR_ASSIGN;
            } else {
                tok = TOK_SHR;
            }
        } else if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_GEQ;
        } else {
            tok = TOK_GT;
        }
        return tok;

    case '=':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_EQ;
        } else {
            tok = TOK_ASSIGN;
        }
        return tok;

    case '!':
        if (*src_ptr == '=') {
            src_ptr++;
            tok = TOK_NEQ;
        } else {
            tok = TOK_NOT;
        }
        return tok;

    default:
        tok = TOK_EOF;
        return tok;
    }
}

const char *token_name(int token) {
    switch (token) {
    case TOK_EOF:
        return "end-of-file";
    case TOK_IDENT:
        return "identifier";
    case TOK_NUM:
        return "number";
    case TOK_STRING:
        return "string literal";
    case TOK_INT:
        return "int";
    case TOK_CHAR:
        return "char";
    case TOK_VOID:
        return "void";
    case TOK_SHORT:
        return "short";
    case TOK_LONG:
        return "long";
    case TOK_UNSIGNED:
        return "unsigned";
    case TOK_RETURN:
        return "return";
    case TOK_IF:
        return "if";
    case TOK_ELSE:
        return "else";
    case TOK_WHILE:
        return "while";
    case TOK_FOR:
        return "for";
    case TOK_DO:
        return "do";
    case TOK_BREAK:
        return "break";
    case TOK_CONTINUE:
        return "continue";
    case TOK_SIZEOF:
        return "sizeof";
    case TOK_PRINTF:
        return "printf";
    case TOK_SYSCALL:
        return "syscall";
    case TOK_LPAREN:
        return "'('";
    case TOK_RPAREN:
        return "')'";
    case TOK_LBRACE:
        return "'{'";
    case TOK_RBRACE:
        return "'}'";
    case TOK_LBRACKET:
        return "'['";
    case TOK_RBRACKET:
        return "']'";
    case TOK_SEMICOLON:
        return "';'";
    case TOK_COMMA:
        return "','";
    case TOK_COLON:
        return "':'";
    case TOK_PLUS:
        return "'+'";
    case TOK_MINUS:
        return "'-'";
    case TOK_STAR:
        return "'*'";
    case TOK_SLASH:
        return "'/'";
    case TOK_MOD:
        return "'%'";
    case TOK_AMP:
        return "'&'";
    case TOK_PIPE:
        return "'|'";
    case TOK_CARET:
        return "'^'";
    case TOK_TILDE:
        return "'~'";
    case TOK_SHL:
        return "'<<'";
    case TOK_SHR:
        return "'>>'";
    case TOK_AND:
        return "'&&'";
    case TOK_OR:
        return "'||'";
    case TOK_NOT:
        return "'!'";
    case TOK_LT:
        return "'<'";
    case TOK_GT:
        return "'>'";
    case TOK_LEQ:
        return "'<='";
    case TOK_GEQ:
        return "'>='";
    case TOK_EQ:
        return "'=='";
    case TOK_NEQ:
        return "'!='";
    case TOK_INC:
        return "'++'";
    case TOK_DEC:
        return "'--'";
    case TOK_ASSIGN:
        return "'='";
    case TOK_ADD_ASSIGN:
        return "'+='";
    case TOK_SUB_ASSIGN:
        return "'-='";
    case TOK_MUL_ASSIGN:
        return "'*='";
    case TOK_DIV_ASSIGN:
        return "'/='";
    case TOK_MOD_ASSIGN:
        return "'%='";
    case TOK_AND_ASSIGN:
        return "'&='";
    case TOK_OR_ASSIGN:
        return "'|='";
    case TOK_XOR_ASSIGN:
        return "'^='";
    case TOK_SHL_ASSIGN:
        return "'<<='";
    case TOK_SHR_ASSIGN:
        return "'>>='";
    default:
        return "unknown token";
    }
}
