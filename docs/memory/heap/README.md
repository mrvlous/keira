<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Heap Allocator & Slab Cache

Dynamic kernel allocation is powered by a segregated free-list allocator and object slab cache.

---

## Heap Index

| Document | Description |
| :--- | :--- |
| [`freelist.md`](freelist.md) | Segregated free-list, size classes (16B to 4096B), canary checks |
| [`slab.md`](slab.md) | Kernel object cache (`kmem_cache`) for fixed-size descriptors |
