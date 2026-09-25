<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Segregated Free-List Heap Allocator

General dynamic allocation (`crates/mem/src/heap/pool/`) uses segregated power-of-two free-lists.

---

## Size Classes

* Classes: 16, 32, 64, 128, 256, 512, 1024, 2048, 4096 bytes.
* Memory Canaries: Guard bytes placed at header and trailer to detect buffer overflows and double frees.
* Expansion: Requests additional 4 KiB pages from the VMM when a free-list pool is exhausted.
