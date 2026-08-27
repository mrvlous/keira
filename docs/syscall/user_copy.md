<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Validated User Pointer Copy Primitives

This document specifies the hardened memory copying functions that transfer data between unprivileged Userland (Ring 3) and privileged Kernel (Ring 0).

---

## Security Invariants

1. **Non-Null Verification**: Pointer must not be `NULL` (`0x0`).
2. **Userland Boundary Check**: Pointer and length range must lie strictly below the kernel canonical base (`0x0000_7FFF_FFFF_FFFF` on 64-bit / `0xC000_0000` on 32-bit).
3. **Arithmetic Overflow Guard**: `addr.checked_add(len)` must not wrap around address space boundaries.

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
