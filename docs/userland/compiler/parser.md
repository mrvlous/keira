<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Recursive Descent Parser (`parser.c`)

This document details grammar analysis, precedence climbing expression parsing, and AST construction in the KCC C compiler.

---

## AST Node Types

```c
typedef enum {
    AST_PROGRAM,
    AST_FUNCTION_DEF,
    AST_VAR_DECL,
    AST_BLOCK,
    AST_RETURN,
    AST_IF_STMT,
    AST_WHILE_STMT,
    AST_FOR_STMT,
    AST_BINARY_OP,
    AST_UNARY_OP,
    AST_FUNC_CALL,
    AST_LITERAL_NUM,
    AST_LITERAL_STR,
    AST_VARIABLE,
} AstNodeType;
```
