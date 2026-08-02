/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_INCLUDE_KEIRA_HEAP_H
#define KEIRA_INCLUDE_KEIRA_HEAP_H

#include <stddef.h>
#include <stdint.h>

/**
 * Kernel Memory Management Subsystem - Heap Allocator Interface
 *
 * Exposes core dynamic memory allocation primitives for kernel memory management.
 */

/**
 * heap_init - Initialize kernel bump allocator memory boundaries.
 * @start: Physical starting address of kernel heap memory block.
 * @size: Total heap size in bytes.
 */
void heap_init(void *start, size_t size);

/**
 * kmalloc - Allocate a contiguous block of heap memory.
 * @size: Minimum number of bytes requested.
 *
 * Return: Pointer to 16-byte aligned memory block, or NULL if out of memory.
 */
void *kmalloc(size_t size);

/**
 * kfree - Free allocated heap memory block.
 * @ptr: Pointer to memory block to release (no-op for bump allocator).
 */
void kfree(void *ptr);

/**
 * heap_get_total - Read total configured heap capacity.
 *
 * Return: Total heap size in bytes.
 */
size_t heap_get_total(void);

/**
 * heap_get_used - Read cumulative bytes allocated.
 *
 * Return: Number of allocated bytes.
 */
size_t heap_get_used(void);

/**
 * heap_get_free - Read remaining unallocated bytes.
 *
 * Return: Number of free bytes remaining in heap.
 */
size_t heap_get_free(void);

/**
 * heap_get_alloc_count - Read total number of allocation requests.
 *
 * Return: Allocation count.
 */
size_t heap_get_alloc_count(void);

/**
 * heap_get_peak - Read peak heap usage in bytes.
 *
 * Return: Peak allocated bytes.
 */
size_t heap_get_peak(void);

#endif /* KEIRA_INCLUDE_KEIRA_HEAP_H */
