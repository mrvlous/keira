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
| `MAP_SHARED` | `0x01` | Share changes across processes |
| `MAP_PRIVATE` | `0x02` | Copy-on-write private mapping |
| `MAP_ANONYMOUS` | `0x20` | Mapping is not backed by any file |

---

## 2. Functions

### `mmap`
```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
```
Allocates a new virtual memory region in the process address space. Returns `MAP_FAILED` on failure.

### `munmap`
```c
int munmap(void *addr, size_t length);
```
Unmaps pages starting at `addr` of size `length`.
