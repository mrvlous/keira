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
#include <syscall.h>

int io_uring_queue_init(uint32_t entries, struct io_uring *ring, uint32_t flags) {
    if (!ring) {
        return -EINVAL;
    }
    memset(ring, 0, sizeof(*ring));
    ring->params.flags = flags;

    int ret = io_uring_setup(entries, &ring->params);
    if (ret < 0) {
        return ret;
    }

    ring->ring_fd = ret;
    ring->sqes = (struct io_uring_sqe *)(uintptr_t)ring->params.sq_off.user_addr;
    ring->cqes = (struct io_uring_cqe *)(uintptr_t)ring->params.cq_off.user_addr;
    ring->sq_head = 0;
    ring->sq_tail = 0;
    ring->cq_head = 0;

    return 0;
}

void io_uring_queue_exit(struct io_uring *ring) {
    if (ring) {
        if (ring->ring_fd >= 0) {
            sys_close(ring->ring_fd);
            ring->ring_fd = -1;
        }
    }
}
