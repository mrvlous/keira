<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Preprocessor (`userland/bin/kcc/preproc/`)

The preprocessor handles macro expansion, header file inclusion, and conditional directives prior to lexical analysis.

---

## Directives Supported

* `#include "header.h"` and `#include <header.h>`: Recursively loads and includes header files from standard include directories (`/include`, `/usr/include`).
* `#define NAME value`: Object-like and function-like macro replacement.
* `#undef NAME`: Cancels macro definitions.
* `#ifdef`, `#ifndef`, `#if`, `#elif`, `#else`, `#endif`: Conditional code emission based on preprocessor expressions.
* `#pragma once`: Header guard optimization preventing redundant inclusions.
