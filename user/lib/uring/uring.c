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
#include <stdlib.h>
#include <string.h>
#include <sys/io_uring.h>
#include <sys/syscall.h>

int io_uring_setup(uint32_t entries, struct io_uring_params *params) {
    return (int)syscall2(SYS_IO_URING_SETUP, (uint64_t)entries, (uint64_t)(uintptr_t)params);
}

int io_uring_enter(int fd, uint32_t to_submit, uint32_t min_complete, uint32_t flags) {
    return (int)syscall4(SYS_IO_URING_ENTER, (uint64_t)fd, (uint64_t)to_submit,
                         (uint64_t)min_complete, (uint64_t)flags);
}

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
    ring->sq_tail = 0;
    ring->cq_head = 0;

    return 0;
}

struct io_uring_sqe *io_uring_get_sqe(struct io_uring *ring) {
    if (!ring || !ring->sqes) {
        return NULL;
    }
    uint32_t mask = ring->params.sq_off.ring_mask;
    if (mask == 0) {
        mask = ring->params.sq_entries - 1;
    }
    struct io_uring_sqe *sqe = &ring->sqes[ring->sq_tail & mask];
    memset(sqe, 0, sizeof(*sqe));
    ring->sq_tail++;
    return sqe;
}

int io_uring_submit(struct io_uring *ring) {
    if (!ring) {
        return -EINVAL;
    }
    uint32_t to_submit = ring->sq_tail;
    ring->sq_tail = 0;
    return io_uring_enter(ring->ring_fd, to_submit, 0, 0);
}

int io_uring_peek_cqe(struct io_uring *ring, struct io_uring_cqe **cqe_ptr) {
    if (!ring || !ring->cqes || !cqe_ptr) {
        return -EINVAL;
    }
    uint32_t mask = ring->params.cq_off.ring_mask;
    if (mask == 0) {
        mask = ring->params.cq_entries - 1;
    }
    *cqe_ptr = &ring->cqes[ring->cq_head & mask];
    return 0;
}

void io_uring_cqe_seen(struct io_uring *ring, struct io_uring_cqe *cqe) {
    (void)cqe;
    if (ring) {
        ring->cq_head++;
    }
}

void io_uring_queue_exit(struct io_uring *ring) {
    if (ring) {
        if (ring->ring_fd >= 0) {
            sys_close(ring->ring_fd);
            ring->ring_fd = -1;
        }
    }
}
