/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_MALLOC_H
#define KEIRA_USER_LIB_MALLOC_H

#include <stddef.h>

/**
 * Keira User-Space Heap Allocator Interface
 */

/**
 * malloc - Allocate a block of memory from the process heap.
 * @size: Requested payload size in bytes.
 *
 * Return: Pointer to allocated payload memory, or NULL on allocation failure.
 */
void *malloc(size_t size);

/**
 * free - Release previously allocated heap memory block.
 * @ptr: Pointer to memory block payload returned by malloc/calloc/realloc.
 */
void free(void *ptr);

/**
 * calloc - Allocate zero-initialized memory array from process heap.
 * @num: Element count.
 * @size: Size of individual element in bytes.
 *
 * Return: Pointer to zero-initialized memory block, or NULL on failure.
 */
void *calloc(size_t num, size_t size);

/**
 * realloc - Resize existing dynamic memory block.
 * @ptr: Pointer to existing memory block payload.
 * @size: New target capacity in bytes.
 *
 * Return: Pointer to resized payload buffer, or NULL on failure.
 */
void *realloc(void *ptr, size_t size);

#endif /* KEIRA_USER_LIB_MALLOC_H */
