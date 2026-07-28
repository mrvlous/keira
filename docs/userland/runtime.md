# User Runtime Library (libc)

This document details the user-space C standard library interfaces, dynamic memory management, and system call wrapper mechanics in Keira Kernel.

## 1. System Call Wrapper Interface
User applications interact with the kernel by executing system calls wrapped in assembly routines ([syscall.h](../../user/include/syscall.h)).

### Assembly Wrappers
System calls are declared using standard wrappers that assign parameters to Registers matching the System V AMD64 ABI:
*   **System Call Number**: Passed in the `RAX` register.
*   **Arguments**: Loaded sequentially into registers `RDI`, `RSI`, `RDX`, `R10`, `R8`, and `R9`.
*   **Trigger**: The `syscall` instruction is executed.
*   **Return Value**: The result is retrieved from the `RAX` register.

---

## 2. Dynamic Memory Management (`malloc`)
The user-space memory allocator ([malloc.h](../../user/include/malloc.h)) manages heap memory allocation for user processes.

### Memory Allocation APIs
*   `void *malloc(size_t size)`: Allocates a block of heap memory.
*   `void free(void *ptr)`: Releases a previously allocated block of memory.
*   `void *calloc(size_t num, size_t size)`: Allocates a zero-initialized block of memory.
*   `void *realloc(void *ptr, size_t size)`: Resizes an existing heap allocation.

### Under the Hood: `sbrk` System Call
The allocator requests raw memory pages from the kernel using the `sbrk` system call:
1.  **Syscall Parameters**: `sbrk` takes a signed increment `increment`.
2.  **Kernel Execution**: The kernel handler ([handler.rs](../../kernel/src/syscall/handler.rs)):
    *   If `increment > 0`, it maps new physical pages in the process's lower-half virtual address space to move the program break forward.
    *   If `increment < 0`, it unmaps pages to shrink the program break.
    *   Returns the previous program break virtual address.

---

## 3. String and Memory Utilities
Standard memory and string copy routines are implemented in [string.h](../../user/include/string.h):
*   `size_t strlen(const char *str)`: Calculates the length of a null-terminated string.
*   `void *memcpy(void *dest, const void *src, size_t n)`: Copies memory blocks.
*   `void *memset(void *s, int c, size_t n)`: Fills memory regions with constant bytes.
*   `int strcmp(const char *s1, const char *s2)`: Compares two strings.
*   `char *strcpy(char *dest, const char *src)`: Copies strings.
*   `char *strncpy(char *dest, const char *src, size_t n)`: Copies up to `n` characters of a string.
