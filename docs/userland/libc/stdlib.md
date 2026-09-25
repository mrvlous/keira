<!-- SPDX-License-Identifier: GPL-2.0-only -->

# General Utilities (`userland/lib/stdlib/`)

The `stdlib` module implements dynamic memory management, process termination, environment queries, and number conversion.

---

## Heap Allocator (`malloc` / `free`)

Userland heap allocation utilizes a two-level architecture:
1. **Page-level allocation**: Requests 4 KB virtual pages from the kernel via `SYS_mmap` or `SYS_brk`.
2. **Chunk-level allocation**: Segregated free-list allocator splits pages into sized buckets with header guards:

```c
typedef struct block_header {
    size_t size;
    int is_free;
    struct block_header *next;
} block_header_t;
```

---

## Utility Functions

* `atoi`, `atol`, `strtol`: Converts string representations to integer primitives.
* `rand`, `srand`: Linear congruential pseudo-random number generator.
* `exit`: Flushes open stdio buffers, executes registered `atexit` callbacks, and terminates process via `SYS_exit`.
* `abort`: Raises `SIGABRT` to produce an immediate process termination and core dump.
