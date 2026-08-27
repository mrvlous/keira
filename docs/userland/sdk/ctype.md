<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Character Classification & Conversion (`ctype.h`)

This document specifies ASCII character classification functions in the Keira Kernel C SDK.

---

## Core API (`user/include/ctype.h` & `user/lib/ctype/`)

```c
int isdigit(int c);
int isalpha(int c);
int isalnum(int c);
int isspace(int c);
int isupper(int c);
int islower(int c);
int isprint(int c);
int tolower(int c);
int toupper(int c);
```
