<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Unsafe Rust Safety Contracts & Invariants

Guidelines for writing safe and correct `unsafe` code in freestanding kernel mode.

## Core Rules

1. **Document `# Safety`**: Every `unsafe fn` and `unsafe` block MUST explain why the caller or invariant guarantees memory safety.
2. **Pointer Validation**: Never dereference raw pointers provided by Ring 3 userland without verifying boundaries against `validate_virt_addr_range()`.
3. **Interrupt Disabling (`cli`)**: When modifying shared global data structures from interrupt handlers or task context, disable CPU interrupts to prevent deadlocks.
4. **Volatile MMIO**: Always use `core::ptr::read_volatile` and `core::ptr::write_volatile` when accessing memory-mapped I/O device registers (e.g. APIC, HPET, NVMe doorbells).
