/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <malloc.h>
#include <string.h>
#include <syscall.h>

#define HEAP_PAGE_SIZE 4096

typedef struct BlockHeader {
    size_t size;
    int is_free;
    struct BlockHeader *next;
} BlockHeader;

#define BLOCK_HEADER_SIZE sizeof(BlockHeader)

static BlockHeader *free_list_head = NULL;
static void *heap_start = NULL;
static void *heap_end = NULL;

static BlockHeader *request_space(size_t size) {
    size_t total_size = size + BLOCK_HEADER_SIZE;
    size_t pages = (total_size + HEAP_PAGE_SIZE - 1) / HEAP_PAGE_SIZE;
    size_t alloc_size = pages * HEAP_PAGE_SIZE;

    void *mem = sys_mmap(NULL, alloc_size, 3, 0x22, -1, 0);
    if (!mem || mem == (void *)-1)
        return NULL;

    BlockHeader *block = (BlockHeader *)mem;
    block->size = alloc_size - BLOCK_HEADER_SIZE;
    block->is_free = 0;
    block->next = NULL;

    if (!heap_start)
        heap_start = mem;
    heap_end = (void *)((uintptr_t)mem + alloc_size);

    return block;
}

void *malloc(size_t size) {
    if (size == 0)
        return NULL;

    size = (size + 7) & ~7; /* 8-byte align */

    BlockHeader *curr = free_list_head;
    BlockHeader *prev = NULL;

    while (curr) {
        if (curr->is_free && curr->size >= size) {
            if (curr->size >= size + BLOCK_HEADER_SIZE + 16) {
                BlockHeader *next_block =
                    (BlockHeader *)((uintptr_t)curr + BLOCK_HEADER_SIZE + size);
                next_block->size = curr->size - size - BLOCK_HEADER_SIZE;
                next_block->is_free = 1;
                next_block->next = curr->next;

                curr->size = size;
                curr->next = next_block;
            }
            curr->is_free = 0;
            return (void *)(curr + 1);
        }
        prev = curr;
        curr = curr->next;
    }

    BlockHeader *block = request_space(size);
    if (!block)
        return NULL;

    if (prev) {
        prev->next = block;
    } else {
        free_list_head = block;
    }

    return (void *)(block + 1);
}

void free(void *ptr) {
    if (!ptr)
        return;

    BlockHeader *block = (BlockHeader *)ptr - 1;
    block->is_free = 1;

    BlockHeader *curr = free_list_head;
    while (curr) {
        if (curr->is_free && curr->next && curr->next->is_free) {
            uintptr_t next_addr = (uintptr_t)curr + BLOCK_HEADER_SIZE + curr->size;
            if (next_addr == (uintptr_t)curr->next) {
                curr->size += BLOCK_HEADER_SIZE + curr->next->size;
                curr->next = curr->next->next;
                continue;
            }
        }
        curr = curr->next;
    }
}

void *calloc(size_t nmemb, size_t size) {
    size_t total = nmemb * size;
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

    BlockHeader *block = (BlockHeader *)ptr - 1;
    if (block->size >= size)
        return ptr;

    void *new_ptr = malloc(size);
    if (new_ptr) {
        memcpy(new_ptr, ptr, block->size);
        free(ptr);
    }
    return new_ptr;
}
