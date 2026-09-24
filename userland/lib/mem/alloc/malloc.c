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
#include <unistd.h>

void *malloc(size_t size) {
    if (size == 0)
        return NULL;

    size = ALIGN16(size);

    /* Tier 1: Large allocation bypass via anonymous mmap */
    if (size > MMAP_THRESHOLD) {
        size_t total_size = ALIGN4K(size + BLOCK_HEADER_SIZE);
        void *mem = sys_mmap(NULL, total_size, 3, 0x22, -1, 0);
        if (!mem || mem == (void *)-1)
            return NULL;

        BlockHeader *block = (BlockHeader *)mem;
        block->size = total_size - BLOCK_HEADER_SIZE;
        block->prev_size = 0;
        block->is_free = 0;
        block->is_mmap = 1;
        block->magic = HEAP_MAGIC;
        block->padding = 0;
        block->next_free = NULL;
        block->prev_free = NULL;
        return (void *)(block + 1);
    }

    /* Tier 2: Small/Medium allocation from process heap */
    BlockHeader *curr = free_list_head;
    while (curr) {
        if (curr->magic == HEAP_MAGIC && curr->is_free && curr->size >= size) {
            remove_from_free_list(curr);

            /* Split if remainder is large enough for header + min payload */
            if (curr->size >= size + BLOCK_HEADER_SIZE + 32) {
                BlockHeader *split = (BlockHeader *)((uintptr_t)curr + BLOCK_HEADER_SIZE + size);
                split->size = curr->size - size - BLOCK_HEADER_SIZE;
                split->prev_size = size;
                split->is_free = 1;
                split->is_mmap = 0;
                split->magic = HEAP_MAGIC;
                split->padding = 0;
                split->next_free = NULL;
                split->prev_free = NULL;

                /* Update following block's prev_size */
                BlockHeader *after =
                    (BlockHeader *)((uintptr_t)split + BLOCK_HEADER_SIZE + split->size);
                if ((uintptr_t)after < (uintptr_t)heap_break && after->magic == HEAP_MAGIC &&
                    !after->is_mmap) {
                    after->prev_size = split->size;
                }

                insert_into_free_list(split);
                curr->size = size;
            }

            curr->is_free = 0;
            return (void *)(curr + 1);
        }
        curr = curr->next_free;
    }

    /* No suitable free block, expand heap via sbrk */
    size_t need = size + BLOCK_HEADER_SIZE;
    size_t alloc_bytes = (need > HEAP_CHUNK_MIN) ? need : HEAP_CHUNK_MIN;
    alloc_bytes = ALIGN4K(alloc_bytes);

    void *p = sbrk((intptr_t)alloc_bytes);
    if (!p || p == (void *)-1)
        return NULL;

    if (!heap_base)
        heap_base = p;

    size_t prev_block_size = 0;
    heap_break = (void *)((uintptr_t)p + alloc_bytes);

    BlockHeader *block = (BlockHeader *)p;
    block->size = alloc_bytes - BLOCK_HEADER_SIZE;
    block->prev_size = prev_block_size;
    block->is_free = 0;
    block->is_mmap = 0;
    block->magic = HEAP_MAGIC;
    block->padding = 0;
    block->next_free = NULL;
    block->prev_free = NULL;

    if (block->size >= size + BLOCK_HEADER_SIZE + 32) {
        BlockHeader *split = (BlockHeader *)((uintptr_t)block + BLOCK_HEADER_SIZE + size);
        split->size = block->size - size - BLOCK_HEADER_SIZE;
        split->prev_size = size;
        split->is_free = 1;
        split->is_mmap = 0;
        split->magic = HEAP_MAGIC;
        split->padding = 0;
        split->next_free = NULL;
        split->prev_free = NULL;

        insert_into_free_list(split);
        block->size = size;
    }

    return (void *)(block + 1);
}
