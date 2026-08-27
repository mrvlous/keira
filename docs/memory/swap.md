<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Swap Subsystem & Frame Eviction

This document specifies the page frame swapping mechanism, swap partition tracking, and LRU page eviction policies in Keira Kernel.

---

## Swap Entry Representation

When a page is swapped out to disk, the page table entry (PTE) present bit is cleared, and the swap offset is encoded into the upper bits of the PTE:

```
63                                       1 0
+---------------------------------------+-+-+
| Swap Slot Index (Sector / Offset)     |0|0|
+---------------------------------------+-+-+
                                         | |
                                         | +-- Present = 0
                                         +---- Swapped = 1
```

---

## Core API (`crates/mem/src/swap/mod.rs`)

```rust
pub fn swap_init(device_id: u32, total_slots: usize) -> Result<(), &'static str>;
pub fn swap_out_page(vaddr: usize, paddr: usize) -> Result<u32, &'static str>;
pub fn swap_in_page(slot: u32, target_frame: usize) -> Result<(), &'static str>;
```
