<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM)

Documentation for physical frame allocation in [`crates/mem/src/pmm/`](../../../crates/mem/src/pmm).

## Architecture
- **Per-Frame Allocation Bitmap**: Tracks individual allocation state (allocated vs free) for physical page frames up to 4 GiB (`1,048,576` frames) using a 128 KiB bitmap in BSS (`ALLOCATION_BITMAP`).
- **Physical Address Limit**: `MAX_PHYS_ADDR_LIMIT` is capped at `4 GiB` (`0x1_0000_0000`). Any RAM regions or frame allocation/deallocation requests beyond 4 GiB are explicitly rejected.
- **Synchronized Critical Sections**: Allocator operations (`alloc_frame`, `free_frame`, `free_contiguous_frames`, stats) are protected by a native RAII spinlock (`PmmGuard`) to ensure thread-safe validation, bitmap modification, and free-list linking as a single atomic transaction.
- **LIFO Free List & Bump Allocation**: Allocates frames from the LIFO free list or advances through parsed Multiboot2 usable regions.
- **Physical Memory Semantics**:
  - `total_memory()` / `total_usable_memory()`: Total usable RAM in bytes (sum of all Multiboot2 type=1 memory segments below 4 GiB).
  - `max_physical_address()`: Highest physical address detected across any memory region in the system memory map (capped at `MAX_PHYS_ADDR_LIMIT`).
- **Atomic Batch Deallocation**: Validates every single frame in a requested contiguous range before reclaiming; if any frame was already freed (e.g. partial overlap or double free) or exceeds 4 GiB, the entire operation is atomically rejected.
- **Page Size**: `4096` bytes (`PAGE_SIZE_4K`).
- **Memory Map Parsing**: Traverses Multiboot2 memory tags during boot to mark available RAM regions while preserving low memory (`0x0` - `0x100000`), kernel code (`1MB`+), initrd archive, and ACPI tables.

## Key APIs
- `alloc_frame() -> Option<u64>`: Atomically allocates a zero-cleared physical 4KB frame address (< 4 GiB) and marks it allocated in the bitmap.
- `free_frame(phys_addr: u64) -> bool`: Atomically returns a 4KB frame to the free list with individual per-frame ownership verification and RAM boundary validation.
- `free_contiguous_frames(start_frame: u64, count: usize) -> bool`: Atomically reclaims contiguous physical frames with native batch linking and pre-validation.
- `is_frame_allocated(frame: u64) -> bool`: Queries whether a physical page frame is currently allocated in the bitmap (explicitly false for >= 4 GiB).
- `is_valid_ram_range(start: u64, size: u64) -> bool`: Validates that a physical range fits entirely inside a single continuous usable RAM region without bridging reserved holes or exceeding 4 GiB.
- `total_memory() -> u64`: Returns total detected usable RAM in bytes.
- `total_usable_memory() -> u64`: Alias for `total_memory()`.
- `max_physical_address() -> u64`: Returns highest physical memory address detected in system memory map.
- `get_stats() -> (u64, u64, u64)`: Returns `(total_usable_bytes, used_bytes, free_bytes)`.
