<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Memory Alignment & Bit Utilities

Documentation for memory alignment routines in [`crates/core/src/mem/align.rs`](../../../crates/core/src/mem/align.rs).

## Functions

### `align_up(addr: u64, align: u64) -> u64`
Rounds up `addr` to the nearest multiple of `align`. `align` must be a power of two.
```rust
assert_eq!(keira_core::mem::align_up(0x1005, 4096), 0x2000);
```

### `align_down(addr: u64, align: u64) -> u64`
Rounds down `addr` to the nearest multiple of `align`.
```rust
assert_eq!(keira_core::mem::align_down(0x1FFF, 4096), 0x1000);
```

### `is_aligned(addr: u64, align: u64) -> bool`
Returns `true` if `addr` satisfies the specified power-of-two alignment.
