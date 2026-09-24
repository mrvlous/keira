/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <string.h>
#include <sys/io_uring.h>

struct io_uring_sqe *io_uring_get_sqe(struct io_uring *ring) {
    if (!ring || !ring->sqes) {
        return NULL;
    }
    uint32_t mask = ring->params.sq_entries > 0 ? (ring->params.sq_entries - 1) : 0;
    struct io_uring_sqe *sqe = &ring->sqes[ring->sq_tail & mask];
    memset(sqe, 0, sizeof(*sqe));
    ring->sq_tail++;
    return sqe;
}

int io_uring_submit(struct io_uring *ring) {
    if (!ring) {
        return -EINVAL;
    }
    uint32_t to_submit = ring->sq_tail - ring->sq_head;
    ring->sq_head = ring->sq_tail;
    return io_uring_enter(ring->ring_fd, to_submit, 0, 0);
}
