<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Memory Management (`<sys/mman.h>`)

The `<sys/mman.h>` header defines page-level virtual memory mapping and protection controls.

---

## 1. Protection & Flag Constants

| Constant | Value | Description |
| :--- | :--- | :--- |
| `PROT_NONE` | `0x0` | Pages may not be accessed |
| `PROT_READ` | `0x1` | Pages may be read |
| `PROT_WRITE` | `0x2` | Pages may be written |
| `PROT_EXEC` | `0x4` | Pages may be executed |
| `MAP_SHARED` | `0x01` | Share changes across processes and flush to disk |
| `MAP_PRIVATE` | `0x02` | Copy-on-write private mapping |
| `MAP_FIXED` | `0x10` | Interpret `addr` exactly without relocation |
| `MAP_ANONYMOUS` | `0x20` | Mapping is not backed by any file (zero-initialized) |
| `MAP_POPULATE` | `0x8000` | Pre-populate page tables eagerly instead of demand paging |
| `MS_ASYNC` | `0x1` | Perform asynchronous cache writeout |
| `MS_INVALIDATE` | `0x2` | Invalidate cached pages |
| `MS_SYNC` | `0x4` | Synchronous disk writeout and cache flush |

---

## 2. Functions

### `mmap`
```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
```
Allocates a new virtual memory region in the process address space. When `MAP_ANONYMOUS` is not specified, creates a file-backed mapping referencing `fd` at byte `offset`. Physical frames are lazily faulted in on first access via Interrupt 14 (`#PF`) demand paging. Returns `MAP_FAILED` (`(void *)-1`) on failure.

### `munmap`
```c
int munmap(void *addr, size_t length);
```
Unmaps pages starting at `addr` of size `length`, trimming or splitting active VMAs and releasing physical frames.

### `mprotect`
```c
int mprotect(void *addr, size_t length, int prot);
```
Changes access protections on an existing mapped region. Splits or merges VMAs as necessary and updates page directory entries with TLB invalidation. Returns `0` on success or `-1` on error.

### `msync`
```c
int msync(void *addr, size_t length, int flags);
```
Synchronizes dirty mapped pages of a file-backed shared mapping (`MAP_SHARED`) with the underlying file on disk. Scans page table entries, flushes modified pages to storage, and clears the hardware dirty bit (`PAGE_DIRTY`). Returns `0` on success or `-1` on error.
