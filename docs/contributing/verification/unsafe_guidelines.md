<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Unsafe Rust Guidelines & Safety Invariants

Because Keira is a freestanding operating system kernel interacting directly with hardware registers, MMU page tables, and CPU contexts, `unsafe` Rust is necessary in specific low-level components. This document establishes rigorous guidelines for declaring and auditing `unsafe` code.

---

## 1. The Safety Contract Standard

Every `unsafe` block or function **MUST** include an explanatory `# Safety` docstring section detailing preconditions and invariants:

```rust
/// Reads an 8-bit byte from an I/O port.
///
/// # Safety
///
/// The caller must ensure that:
/// 1. `port` corresponds to an authorized, valid hardware I/O address.
/// 2. Concurrent access to the same I/O port does not violate hardware state machines.
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}
```

---

## 2. Unsafe Review Checklist

During code review, verify:
* **Pointer Validity**: Raw pointers must be checked for non-nullness, proper alignment, and valid page mapping before dereferencing.
* **Aliasing Rules**: Never create multiple mutable references (`&mut T`) to the same memory location, even in kernel space.
* **Interrupt Safety**: Critical hardware sections that manipulate shared memory structures must disable interrupts (`cli`) or hold appropriate spinlocks.
* **Ring 3 Isolation**: Never trust userland pointers. User addresses must always pass through `validate_user_ptr()` before access.
