/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Keira User-Space Heap Allocator Implementation
 *
 * Implements a dynamic memory allocator utilizing `sys_sbrk()` system call.
 * Uses a boundary-tag header structure with first-fit allocation and adjacent
 * free block coalescing to reduce internal and external heap fragmentation.
 */

#include "malloc.h"

#include "string.h"
#include "syscall.h"

/**
 * struct block_header - Dynamic heap chunk descriptor header.
 * @size: Size of usable payload buffer (low bit indicates allocation state: 1=used, 0=free).
 */
struct block_header {
    size_t size;
};

#define BLOCK_HEADER_SIZE sizeof(struct block_header)
#define ALIGNMENT 8
#define ALIGN(size) (((size) + (ALIGNMENT - 1)) & ~(ALIGNMENT - 1))

static void *heap_start = NULL;

/**
 * malloc - Allocate a block of memory from the process heap.
 * @size: Requested payload size in bytes.
 *
 * Return: Pointer to allocated payload memory, or NULL on allocation failure.
 */
void *malloc(size_t size) {
    if (size == 0) {
        return NULL;
    }

    size_t aligned_size = ALIGN(size);

    if (heap_start == NULL) {
        heap_start = sys_sbrk(0);
        if (heap_start == (void *)-1) {
            return NULL;
        }
    }

    void *current_break = sys_sbrk(0);
    char *curr = (char *)heap_start;

    /* First-fit free block search */
    while (curr < (char *)current_break) {
        struct block_header *header = (struct block_header *)curr;
        size_t block_size = header->size & ~1;
        int is_allocated = header->size & 1;

        if (!is_allocated && block_size >= aligned_size) {
            /* Split block if excess space is sufficient for header + alignment */
            if (block_size >= aligned_size + BLOCK_HEADER_SIZE + ALIGNMENT) {
                struct block_header *next_header =
                    (struct block_header *)(curr + BLOCK_HEADER_SIZE + aligned_size);
                next_header->size = (block_size - aligned_size - BLOCK_HEADER_SIZE) & ~1;
                header->size = (aligned_size | 1);
            } else {
                header->size |= 1;
            }
            return (void *)(curr + BLOCK_HEADER_SIZE);
        }
        curr += BLOCK_HEADER_SIZE + block_size;
    }

    /* Request additional program break space from kernel */
    size_t required = BLOCK_HEADER_SIZE + aligned_size;
    void *prev_break = sys_sbrk((long)required);
    if (prev_break == (void *)-1) {
        return NULL;
    }

    struct block_header *header = (struct block_header *)prev_break;
    header->size = (aligned_size | 1);

    return (void *)((char *)prev_break + BLOCK_HEADER_SIZE);
}

/**
 * free - Release previously allocated heap memory block.
 * @ptr: Pointer to memory block payload returned by malloc/calloc/realloc.
 */
void free(void *ptr) {
    if (ptr == NULL) {
        return;
    }

    struct block_header *header = (struct block_header *)((char *)ptr - BLOCK_HEADER_SIZE);
    header->size &= ~1;

    if (heap_start == NULL) {
        return;
    }

    void *current_break = sys_sbrk(0);
    char *curr = (char *)heap_start;

    /* Coalesce contiguous unallocated heap blocks */
    while (curr < (char *)current_break) {
        struct block_header *current_header = (struct block_header *)curr;
        size_t current_size = current_header->size & ~1;
        int current_allocated = current_header->size & 1;

        if (!current_allocated) {
            char *next = curr + BLOCK_HEADER_SIZE + current_size;
            if (next < (char *)current_break) {
                struct block_header *next_header = (struct block_header *)next;
                size_t next_size = next_header->size & ~1;
                int next_allocated = next_header->size & 1;

                if (!next_allocated) {
                    current_header->size = (current_size + BLOCK_HEADER_SIZE + next_size) & ~1;
                    continue;
                }
            }
        }
        curr += BLOCK_HEADER_SIZE + current_size;
    }
}

/**
 * calloc - Allocate zero-initialized memory array from process heap.
 * @num: Element count.
 * @size: Size of individual element in bytes.
 *
 * Return: Pointer to zero-initialized memory block, or NULL on failure.
 */
void *calloc(size_t num, size_t size) {
    size_t total = num * size;
    void *ptr = malloc(total);
    if (ptr != NULL) {
        memset(ptr, 0, total);
    }
    return ptr;
}

/**
 * realloc - Resize existing dynamic memory block.
 * @ptr: Pointer to existing memory block payload.
 * @size: New target capacity in bytes.
 *
 * Return: Pointer to resized payload buffer, or NULL on failure.
 */
void *realloc(void *ptr, size_t size) {
    if (ptr == NULL) {
        return malloc(size);
    }
    if (size == 0) {
        free(ptr);
        return NULL;
    }

    struct block_header *header = (struct block_header *)((char *)ptr - BLOCK_HEADER_SIZE);
    size_t current_size = header->size & ~1;
    if (current_size >= size) {
        return ptr;
    }

    void *new_ptr = malloc(size);
    if (new_ptr != NULL) {
        memcpy(new_ptr, ptr, current_size);
        free(ptr);
    }
    return new_ptr;
}