// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
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

#include <errno.h>
#include <sys/io_uring.h>

int io_uring_peek_cqe(struct io_uring *ring, struct io_uring_cqe **cqe_ptr) {
    if (!ring || !ring->cqes || !cqe_ptr) {
        return -EINVAL;
    }
    uint32_t mask = ring->params.cq_entries > 0 ? (ring->params.cq_entries - 1) : 0;
    *cqe_ptr = &ring->cqes[ring->cq_head & mask];
    return 0;
}

void io_uring_cqe_seen(struct io_uring *ring, struct io_uring_cqe *cqe) {
    (void)cqe;
    if (ring) {
        ring->cq_head++;
    }
}
