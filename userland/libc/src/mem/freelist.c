/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#include "heap.h"

BlockHeader *free_list_head = NULL;
void *heap_base = NULL;
void *heap_break = NULL;

void remove_from_free_list(BlockHeader *block) {
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

void insert_into_free_list(BlockHeader *block) {
    if (!block)
        return;
    block->next_free = free_list_head;
    block->prev_free = NULL;
    if (free_list_head)
        free_list_head->prev_free = block;
    free_list_head = block;
}
