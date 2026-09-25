<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Segregated Free-List Heap Allocator

General dynamic allocation (`crates/mem/src/heap/pool/`) uses segregated power-of-two free-lists.

---

## Size Classes

* Classes: 16, 32, 64, 128, 256, 512, 1024, 2048, 4096 bytes.
* Memory Canaries: Guard bytes placed at header and trailer to detect buffer overflows and double frees.
* Expansion: Requests additional 4 KiB pages from the VMM when a free-list pool is exhausted.

---

## Telemetry & Capacity Accounting

The kernel heap metrics engine (`crates/mem/src/heap/pool/stats.rs`) tracks allocation patterns and fragmentation:
* `heap_get_telemetry()`: Computes total capacity, active allocated bytes, unallocated free bytes, peak usage watermark, active block count, and fragmentation percentage against arena allocations.
* Displayed in userspace via the `memory` shell command.

---

## Bare-Metal Stress Testing

The segregated free-list allocator includes an in-kernel automated stress tester (`heap_stress_test()`):
* Allocates 32 mixed-size blocks across all primary size classes (16B–512B) with byte pattern verification.
* Exercises partial, non-contiguous deallocation (odd/even alternating release) to populate segregated free-lists.
* Verifies subsequent allocations properly recycle freed blocks without requesting new arena pages.
* Asserts total zero-leak reclamation upon test completion.
* Executable directly from the interactive shell with `memory -t` or `memory --test`.
