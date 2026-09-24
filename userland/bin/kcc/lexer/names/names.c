/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "lexer.h"

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
    case TOK_SIGNED:
        return "signed";
    case TOK_CONST:
        return "const";
    case TOK_EXTERN:
        return "extern";
    case TOK_DOT:
        return "'.'";
    case TOK_ELLIPSIS:
        return "'...'";
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
