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
 * Kernel Memory Management - Bump Heap Allocator Implementation
 *
 * Provides a sequential bump allocator for early kernel boot memory requests.
 * All memory allocations are rounded up to 16-byte alignment boundaries to
 * satisfy x86_64 SIMD and ABI requirements.
 */

#include "../include/keira/heap.h"

#include <stddef.h>
#include <stdint.h>

#define HEAP_ALIGNMENT 16
#define HEAP_ALIGN_MASK (HEAP_ALIGNMENT - 1)

static uint8_t *heap_start = NULL;
static uint8_t *heap_end = NULL;
static uint8_t *heap_next = NULL;

/**
 * heap_init - Initialize kernel bump allocator memory boundaries.
 * @start: Physical starting address of kernel heap memory block.
 * @size: Total heap size in bytes.
 */
void heap_init(void *start, size_t size) {
    /* Set physical starting and ending boundaries of C kernel heap */
    heap_start = (uint8_t *)start;
    heap_end = heap_start + size;
    heap_next = heap_start;
}

/**
 * kmalloc - Allocate a contiguous block of heap memory.
 * @size: Minimum number of bytes requested.
 *
 * Return: Pointer to 16-byte aligned memory block, or NULL if out of memory.
 */
void *kmalloc(size_t size) {
    /* Return NULL if zero bytes requested */
    if (size == 0) {
        return NULL;
    }

    /* Round up allocation request to 16-byte boundary alignment */
    size = (size + HEAP_ALIGN_MASK) & ~((size_t)HEAP_ALIGN_MASK);

    /* Verify allocation fits within remaining heap limit */
    if (heap_next + size > heap_end) {
        return NULL;
    }

    /* Save current allocation pointer and bump pointer forward */
    void *ptr = heap_next;
    heap_next += size;
    return ptr;
}

/**
 * kfree - Free allocated heap memory block.
 * @ptr: Pointer to memory block to release (no-op for bump allocator).
 */
void kfree(void *ptr) {
    /* Bump allocator does not reclaim individual blocks until full reset */
    (void)ptr;
}

/**
 * heap_get_total - Read total configured heap capacity.
 *
 * Return: Total heap size in bytes.
 */
size_t heap_get_total(void) {
    return (size_t)(heap_end - heap_start);
}

/**
 * heap_get_used - Read cumulative bytes allocated.
 *
 * Return: Number of allocated bytes.
 */
size_t heap_get_used(void) {
    return (size_t)(heap_next - heap_start);
}

/**
 * heap_get_free - Read remaining unallocated bytes.
 *
 * Return: Number of free bytes remaining in heap.
 */
size_t heap_get_free(void) {
    return (size_t)(heap_end - heap_next);
}
