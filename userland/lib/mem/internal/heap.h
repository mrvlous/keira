/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KEIRA_LIB_MEM_HEAP_H
#define _KEIRA_LIB_MEM_HEAP_H

#include <stddef.h>
#include <stdint.h>

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

extern BlockHeader *free_list_head;
extern void *heap_base;
extern void *heap_break;

void remove_from_free_list(BlockHeader *block);
void insert_into_free_list(BlockHeader *block);

#endif /* _KEIRA_LIB_MEM_HEAP_H */
