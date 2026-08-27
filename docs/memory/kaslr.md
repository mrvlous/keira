<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Address Space Layout Randomization (KASLR)

This document describes the entropy generation, virtual base relocation, and page table randomization implemented in Keira Kernel.

---

## Randomization Design

Keira Kernel generates a dynamic 2MB-aligned virtual memory offset during early bootstrap using hardware entropy sources:

* **Hardware RDRAND Instruction** (when supported by CPUID).
* **Hardware Timestamp Counter (RDTSC)** combined with PIT clock jitter.

```
Base Virtual Address : 0xFFFF800000000000 + (Entropy & 0x1FFFF) * 0x200000
```
