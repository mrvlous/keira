<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM)

Documentation for physical frame allocation in [`crates/mem/src/pmm/`](../../../crates/mem/src/pmm).

## Architecture
- **Bitmap Allocator**: Tracks physical memory frames (4096 bytes per page).
- **Page Size**: `4096` bytes (`PAGE_SIZE_4K`).
- **Memory Map Parsing**: Traverses Multiboot2 memory tags during boot to mark available RAM regions while preserving low memory (`0x0` - `0x100000`), kernel code (`1MB`+), initrd archive, and ACPI tables.

## Key APIs
- `alloc_frame() -> Option<u64>`: Allocates a free physical 4KB frame address.
- `free_frame(phys_addr: u64)`: Returns a frame to the free bitmap.
- `get_stats() -> (u64, u64)`: Returns `(used_pages, total_pages)`.
