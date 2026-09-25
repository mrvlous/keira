<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Page Fault Handling & Demand Paging

Hardware page faults (`Vector 14 / #PF`) are trapped and resolved by the kernel.

---

## Fault Resolution

1. Read faulting linear address from CPU register `CR2`.
2. Inspect the active process VMA list to verify if the address falls within a valid mapping.
3. If valid, allocate a physical frame from the PMM, map it into the page table, and resume execution.
4. For user stack auto-growth within the stack boundary, newly allocated pages enforce $W \oplus X$ protections (`PAGE_NO_EXECUTE` on x86_64) preventing stack-based code injection.
5. If invalid or violating page protections, deliver a `SIGSEGV` signal to the offending process.
