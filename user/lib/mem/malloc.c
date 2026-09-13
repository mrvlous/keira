/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <malloc.h>
#include <stdint.h>
#include <string.h>
#include <syscall.h>
#include <unistd.h>

#define HEAP_MAGIC 0x5A4B4549U /* Magic canary "KEI" */
#define MMAP_THRESHOLD 131072U /* 128 KiB threshold for mmap tier */
#define HEAP_CHUNK_MIN 65536U  /* 64 KiB min expansion for sbrk */

#define ALIGN16(x) (((x) + 15U) & ~15U)
#define ALIGN4K(x) (((x) + 4095U) & ~4095U)

typedef struct BlockHeader {
    size_t size;      /* Payload capacity in bytes */
    size_t prev_size; /* Size of preceding contiguous heap block */
    uint32_t is_free; /* 1 if block is free, 0 if allocated */
    uint32_t is_mmap; /* 1 if allocated via sys_mmap, 0 if heap sbrk */
    uint32_t magic;   /* HEAP_MAGIC canary */
    uint32_t padding; /* 16-byte alignment padding */
    struct BlockHeader *next_free;
    struct BlockHeader *prev_free;
} BlockHeader;

#define BLOCK_HEADER_SIZE sizeof(BlockHeader)

static BlockHeader *free_list_head = NULL;
static void *heap_base = NULL;
static void *heap_break = NULL;

static void remove_from_free_list(BlockHeader *block) {
    if (!block)
        return;
    if (block->prev_free)
        block->prev_free->next_free = block->next_free;
    else
        free_list_head = block->next_free;

    if (block->next_free)
        block->next_free->prev_free = block->prev_free;

    block->next_free = NULL;
    block->prev_free = NULL;
}

static void insert_into_free_list(BlockHeader *block) {
    if (!block)
        return;
    block->next_free = free_list_head;
    block->prev_free = NULL;
    if (free_list_head)
        free_list_head->prev_free = block;
    free_list_head = block;
}

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
    if (heap_break && (uintptr_t)p == (uintptr_t)heap_break) {
        /* Contiguous with previous break */
        BlockHeader *last = (BlockHeader *)((uintptr_t)p - BLOCK_HEADER_SIZE);
        /* If valid last block, we could record prev_size */
        (void)last;
    }

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
