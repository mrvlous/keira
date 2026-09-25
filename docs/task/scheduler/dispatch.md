<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Context Switching & Register Preservation

Context switching transfers execution between tasks:
1. Save volatile and callee-saved registers onto the current task stack.
2. Update the TSS `RSP0` / `ESP0` pointer with the target task kernel stack.
3. Switch the active address space by reloading register `CR3`.
4. Restore registers and return via `IRETQ` / `IRETD`.
