<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Code Generator (`codegen.c`)

This document details x86_64 machine instruction synthesis and stack frame management in the KCC C compiler.

---

## Code Generation Strategy

* **Stack Frame Setup**:
  ```nasm
  push rbp
  mov rbp, rsp
  sub rsp, <local_vars_size>
  ```
* **Expression Evaluation**: Computes sub-expressions on the hardware evaluation stack (`RAX` / `RBX`).
* **Function Calling**: Emits parameter assignments into `RDI`, `RSI`, `RDX`, `RCX`, `R8`, `R9` conforming to System V AMD64 ABI.
