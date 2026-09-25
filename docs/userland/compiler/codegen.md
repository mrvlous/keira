<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Machine Code Generator (`userland/bin/kcc/codegen/`)

Translates the AST into native x86 machine instructions.

---

## Code Generation Strategy

* **Stack Frame Management**: Standard function prologue (`push %ebp; mov %esp, %ebp; sub $N, %esp`) and epilogue (`mov %ebp, %esp; pop %ebp; ret`).
* **Calling Convention**: System V x86 32-bit calling convention (arguments pushed right-to-left onto stack; return values in `%eax`).
* **Expression Evaluation**: Stack-based intermediate evaluation utilizing accumulator registers (`%eax`, `%edx`, `%ecx`).
