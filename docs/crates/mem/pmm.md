<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM)

Documentation for physical frame allocation in [`crates/mem/src/pmm/`](../../../crates/mem/src/pmm).

## Architecture
- **Bitmap Allocator**: Tracks physical memory frames (4096 bytes per page).
- **Page Size**: `4096` bytes (`PAGE_SIZE_4K`).
- **Memory Map Parsing**: Traverses Multiboot2 memory tags during boot to mark available RAM regions while preserving low memory (`0x0` - `0x100000`), kernel code (`1MB`+), initrd archive, and ACPI tables.

## Key APIs
- `alloc_frame() -> Option<u64>`: Allocates a zero-cleared physical 4KB frame address.
- `free_frame(phys_addr: u64) -> bool`: Returns a 4KB frame to the free list with double-free and RAM boundary validation.
- `free_contiguous_frames(start_frame: u64, count: usize) -> bool`: Reclaims contiguous physical frames with native batch linking and ownership verification.
- `is_valid_ram_range(start: u64, size: u64) -> bool`: Validates that a physical range fits entirely inside a single continuous usable RAM region without bridging reserved holes.
- `total_memory() -> u64`: Returns total detected usable RAM in bytes.
- `total_usable_memory() -> u64`: Alias for `total_memory()`.
- `max_physical_address() -> u64`: Returns highest physical memory address detected in system memory map.
- `get_stats() -> (u64, u64, u64)`: Returns `(total_usable_bytes, used_bytes, free_bytes)`.
