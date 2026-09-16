<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Validated User Pointer Copy Primitives

This document specifies the hardened memory copying functions that transfer data between unprivileged Userland (Ring 3) and privileged Kernel (Ring 0).

---

## Security Invariants

1. **Lower Boundary Check**: Pointer must be `>= 0x10000` to prevent null pointer dereferences and zero-page exploitation.
2. **Userland Boundary Check**: Pointer and length range must lie strictly within canonical userland (`[0x10000, 0x0000_7FFF_FFFF_FFFF]` on 64-bit `x86_64` / `[0x10000, 0xBFFF_FFFF]` on 32-bit `i686`). Any access attempting to touch kernel space (`>= 0xC000_0000` on 32-bit or `>= 0x8000_0000_0000` / high canonical half `0xFFFF_8000_0000_0000` on 64-bit) immediately fails with `-EFAULT` (14).
3. **Arithmetic Overflow Guard**: `addr.checked_add(len)` must not wrap around address space boundaries.
4. **Demand-Paged VMA & Heap Awareness**: If a target page is not yet mapped into active PML4 page tables, `validate_user_ptr` verifies whether the page resides within an active VMA with matching access permissions or within the task heap (`program_break`). Valid demand pages are populated eagerly during pointer validation, preventing kernel-mode `#PF` panics during subsequent `copy_to_user` or `copy_from_user` routines.

---

## Core API (`crates/syscall/src/user_copy/mod.rs`)

```rust
/// Validate user pointer boundaries.
pub unsafe fn validate_user_ptr(ptr: u64, len: u64, is_write: bool) -> Result<(), u64>;

/// Safely copy data from userland memory into a kernel buffer.
pub fn copy_from_user(dst: &mut [u8], src_user_ptr: u64) -> Result<(), u64>;

/// Safely copy data from a kernel buffer into userland memory.
pub fn copy_to_user(dst_user_ptr: u64, src: &[u8]) -> Result<(), u64>;

/// Safely read a null-terminated string from userland memory.
pub fn read_user_string(src_user_ptr: u64, max_len: usize, out_buf: &mut [u8]) -> Result<usize, u64>;
```
