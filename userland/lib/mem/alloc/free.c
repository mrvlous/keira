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
#include <syscall.h>

void free(void *ptr) {
    if (!ptr)
        return;

    BlockHeader *block = (BlockHeader *)ptr - 1;
    if (block->magic != HEAP_MAGIC || block->is_free)
        return;

    /* Release mmap tier allocation immediately */
    if (block->is_mmap) {
        size_t total_size = block->size + BLOCK_HEADER_SIZE;
        sys_munmap((void *)block, total_size);
        return;
    }

    block->is_free = 1;

    /* Forward coalescing with next adjacent heap block */
    BlockHeader *next = (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + block->size);
    if ((uintptr_t)next < (uintptr_t)heap_break && next->magic == HEAP_MAGIC && next->is_free &&
        !next->is_mmap) {
        remove_from_free_list(next);
        block->size += BLOCK_HEADER_SIZE + next->size;

        BlockHeader *after = (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + block->size);
        if ((uintptr_t)after < (uintptr_t)heap_break && after->magic == HEAP_MAGIC &&
            !after->is_mmap) {
            after->prev_size = block->size;
        }
    }

    /* Backward coalescing with previous adjacent heap block */
    if (block->prev_size > 0) {
        BlockHeader *prev =
            (BlockHeader *)((uintptr_t)block - BLOCK_HEADER_SIZE - block->prev_size);
        if ((uintptr_t)prev >= (uintptr_t)heap_base && prev->magic == HEAP_MAGIC && prev->is_free &&
            !prev->is_mmap) {
            remove_from_free_list(prev);
            prev->size += BLOCK_HEADER_SIZE + block->size;
            block = prev;

            BlockHeader *after =
                (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + block->size);
            if ((uintptr_t)after < (uintptr_t)heap_break && after->magic == HEAP_MAGIC &&
                !after->is_mmap) {
                after->prev_size = block->size;
            }
        }
    }

    insert_into_free_list(block);

    BlockHeader *after = (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + block->size);
    if ((uintptr_t)after < (uintptr_t)heap_break && after->magic == HEAP_MAGIC && !after->is_mmap) {
        after->prev_size = block->size;
    }
}
