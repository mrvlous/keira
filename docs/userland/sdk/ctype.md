<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Character Classification & Conversion (`<ctype.h>`)

This document specifies ASCII character validation, classification, and case conversion functions in the Keira C SDK.

---

## Character Classification Table

| Function | Condition | Description |
| :--- | :--- | :--- |
| `int isalpha(int c)` | `(c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')` | Alphabetic character |
| `int isdigit(int c)` | `c >= '0' && c <= '9'` | Decimal digit |
| `int isalnum(int c)` | `isalpha(c) || isdigit(c)` | Alphanumeric character |
| `int isspace(int c)` | `c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v'` | Whitespace character |
| `int isprint(int c)` | `c >= 0x20 && c <= 0x7E` | Printable character |
| `int toupper(int c)` | `c >= 'a' && c <= 'z' ? c - 32 : c` | Convert to uppercase |
| `int tolower(int c)` | `c >= 'A' && c <= 'Z' ? c + 32 : c` | Convert to lowercase |

---

## Core API (`user/sdk/libc/ctype.c`)

```c
#include <ctype.h>

int isalpha(int c);
int isdigit(int c);
int isalnum(int c);
int isspace(int c);
int isupper(int c);
int islower(int c);
int toupper(int c);
int tolower(int c);
```
