<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Low-Level Memory Management (`userland/lib/mem/`)

Direct virtual memory manipulation interfaces interacting with the kernel VMM.

---

## Memory Mapping (`mmap` / `munmap`)

```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
int munmap(void *addr, size_t length);
```

* `prot`: `PROT_READ`, `PROT_WRITE`, `PROT_EXEC`, `PROT_NONE`.
* `flags`: `MAP_SHARED`, `MAP_PRIVATE`, `MAP_ANONYMOUS`, `MAP_FIXED`.
* Anonymous mappings provide the backing memory for userland heap expanders and thread stacks.
