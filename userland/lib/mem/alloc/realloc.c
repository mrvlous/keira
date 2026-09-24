/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "../internal/heap.h"

#include <malloc.h>
#include <string.h>

void *calloc(size_t nmemb, size_t size) {
    if (nmemb == 0 || size == 0)
        return NULL;

    size_t total = nmemb * size;
    if (total / nmemb != size)
        return NULL; /* Overflow protection */

    void *ptr = malloc(total);
    if (ptr)
        memset(ptr, 0, total);
    return ptr;
}

void *realloc(void *ptr, size_t size) {
    if (!ptr)
        return malloc(size);

    if (size == 0) {
        free(ptr);
        return NULL;
    }

    size = ALIGN16(size);
    BlockHeader *block = (BlockHeader *)ptr - 1;
    if (block->magic != HEAP_MAGIC)
        return NULL;

    if (block->size >= size)
        return ptr;

    /* If not mmap, attempt in-place forward expansion */
    if (!block->is_mmap) {
        BlockHeader *next = (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + block->size);
        if ((uintptr_t)next < (uintptr_t)heap_break && next->magic == HEAP_MAGIC && next->is_free &&
            !next->is_mmap) {
            if (block->size + BLOCK_HEADER_SIZE + next->size >= size) {
                remove_from_free_list(next);
                block->size += BLOCK_HEADER_SIZE + next->size;

                if (block->size >= size + BLOCK_HEADER_SIZE + 32) {
                    BlockHeader *split =
                        (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + size);
                    split->size = block->size - size - BLOCK_HEADER_SIZE;
                    split->prev_size = size;
                    split->is_free = 1;
                    split->is_mmap = 0;
                    split->magic = HEAP_MAGIC;
                    split->padding = 0;
                    split->next_free = NULL;
                    split->prev_free = NULL;

                    BlockHeader *after =
                        (BlockHeader *)((uintptr_t)split + BLOCK_HEADER_SIZE + split->size);
                    if ((uintptr_t)after < (uintptr_t)heap_break && after->magic == HEAP_MAGIC &&
                        !after->is_mmap) {
                        after->prev_size = split->size;
                    }

                    insert_into_free_list(split);
                    block->size = size;
                }
                return ptr;
            }
        }
    }

    void *new_ptr = malloc(size);
    if (new_ptr) {
        memcpy(new_ptr, ptr, block->size);
        free(ptr);
    }
    return new_ptr;
}
