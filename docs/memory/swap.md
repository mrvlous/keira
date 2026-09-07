<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Swap Subsystem & Virtual Memory Disk Pager

This document specifies the page frame swapping mechanism, swap partition tracking, bitmap allocation, and disk pager in Keira Kernel.

---

## 1. Architecture & Bitmap Allocator

Keira Kernel implements a bare-metal virtual memory disk swap manager with bitmap-based slot tracking:

* **Capacity**: 64 MB virtual memory swap space backing up to **16,384 4KB page slots**.
* **Bitmap Layout**: Backed by `SWAP_BITMAP: [u64; 256]` (256 words × 64 bits = 16,384 bits).
* **Slot Tracking**: Each bit represents the allocation state of one 4KB disk page slot (`0` = free, `1` = allocated).
* **Backing Devices**: Configurable runtime swap space pointing to `/data/swapfile` or persistent partition nodes (e.g. `/dev/sda2`).

---

## 2. Swap Entry Representation

When a page is swapped out to disk, the page table entry (PTE) present bit is cleared, and the swap offset is encoded into the upper bits of the PTE:

```text
63                                       1 0
+---------------------------------------+-+-+
| Swap Slot Index (Sector / Offset)     |0|0|
+---------------------------------------+-+-+
                                         | |
                                         | +-- Present = 0
                                         +---- Swapped = 1
```

---

## 3. Core API (`crates/mem/src/swap/pager.rs`)

```rust
/// Activate kernel disk swap partition on the specified path.
pub fn swapon(path: &str, swapflags: i32) -> Result<(), &'static str>;

/// Deactivate active swap space and reset allocated slots.
pub fn swapoff(path: Option<&str>) -> Result<(), &'static str>;

/// Allocate a free 4KB swap slot from the active swap pool.
pub fn alloc_swap_slot() -> Option<usize>;

/// Release a previously allocated swap slot back to the free pool.
pub fn free_swap_slot(slot: usize) -> Result<(), &'static str>;

/// Query real-time swap manager telemetry and metrics.
pub fn swap_stats() -> SwapStats;
```

---

## 4. Shell Integration

The `swap` command provides interactive inspection and testing of the swap pager:

```bash
keira> swap on /data/swapfile
Swap Activation: [OK]
  Backing Device : /data/swapfile
  Capacity       : 64 MB (16384 4KB slots)

keira> swap status
Virtual Memory Disk Swap Status: [ACTIVE]
  Backing Device : /data/swapfile
  Total Space    : 64 MB (16384 pages)
  Used Space     : 0 KB (0 pages)
  Free Space     : 64 MB (16384 pages)
  Swap Activity  : 0 In | 0 Out

keira> swap test
[SWAP TEST] Running swap allocator diagnostics...
  Allocated Slot 1 : index 0
  Allocated Slot 2 : index 1
  Allocated Slot 3 : index 2
  Recycled Slot 2  : verified index 1
[SWAP TEST] All 3 slots verified and released [OK]
```
