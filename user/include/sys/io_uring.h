/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYS_IO_URING_H
#define _SYS_IO_URING_H

#include <stdint.h>
#include <sys/types.h>

/* Setup flags */
#define IORING_SETUP_IOPOLL (1U << 0)
#define IORING_SETUP_SQPOLL (1U << 1)
#define IORING_SETUP_SQ_AFF (1U << 2)
#define IORING_SETUP_CQSIZE (1U << 3)
#define IORING_SETUP_CLAMP (1U << 4)
#define IORING_SETUP_ATTACH_WQ (1U << 5)

/* Kernel feature flags */
#define IORING_FEAT_SINGLE_MMAP (1U << 0)
#define IORING_FEAT_NODROP (1U << 1)
#define IORING_FEAT_SUBMIT_STABLE (1U << 2)
#define IORING_FEAT_RW_CUR_POS (1U << 3)

/* Enter flags */
#define IORING_ENTER_GETEVENTS (1U << 0)
#define IORING_ENTER_SQ_WAKEUP (1U << 1)

/* Submission Queue Entry Opcodes */
#define IORING_OP_NOP 0
#define IORING_OP_READV 1
#define IORING_OP_WRITEV 2
#define IORING_OP_FSYNC 3
#define IORING_OP_READ_FIXED 4
#define IORING_OP_WRITE_FIXED 5
#define IORING_OP_POLL_ADD 6
#define IORING_OP_POLL_REMOVE 7
#define IORING_OP_SYNC_FILE_RANGE 8
#define IORING_OP_SENDMSG 9
#define IORING_OP_RECVMSG 10
#define IORING_OP_TIMEOUT 11
#define IORING_OP_TIMEOUT_REMOVE 12
#define IORING_OP_ACCEPT 13
#define IORING_OP_ASYNC_CANCEL 14
#define IORING_OP_LINK_TIMEOUT 15
#define IORING_OP_CONNECT 16
#define IORING_OP_FALLOCATE 17
#define IORING_OP_OPENAT 18
#define IORING_OP_CLOSE 19
#define IORING_OP_FILES_UPDATE 20
#define IORING_OP_STATX 21
#define IORING_OP_READ 22
#define IORING_OP_WRITE 23

/* Linux ABI-compatible Submission Queue Entry (64 bytes) */
struct io_uring_sqe {
    uint8_t opcode;
    uint8_t flags;
    uint16_t ioprio;
    int32_t fd;
    uint64_t off;
    uint64_t addr;
    uint32_t len;
    uint32_t rw_flags;
    uint64_t user_data;
    uint16_t buf_index;
    uint16_t personality;
    int32_t splice_fd_in;
    uint64_t pad2[2];
};

/* Linux ABI-compatible Completion Queue Entry (16 bytes) */
struct io_uring_cqe {
    uint64_t user_data;
    int32_t res;
    uint32_t flags;
};

/* Ring buffer offsets */
struct io_sqring_offsets {
    uint32_t head;
    uint32_t tail;
    uint32_t ring_mask;
    uint32_t ring_entries;
    uint32_t flags;
    uint32_t dropped;
    uint32_t array;
    uint32_t resv1;
    uint64_t user_addr;
    uint64_t pad[2];
};

struct io_cqring_offsets {
    uint32_t head;
    uint32_t tail;
    uint32_t ring_mask;
    uint32_t ring_entries;
    uint32_t overflow;
    uint32_t cqes;
    uint32_t flags;
    uint32_t resv1;
    uint64_t user_addr;
    uint64_t pad[2];
};

/* Parameter structure for io_uring_setup */
struct io_uring_params {
    uint32_t sq_entries;
    uint32_t cq_entries;
    uint32_t flags;
    uint32_t sq_thread_cpu;
    uint32_t sq_thread_idle;
    uint32_t features;
    uint32_t wq_fd;
    uint32_t resv[3];
    struct io_sqring_offsets sq_off;
    struct io_cqring_offsets cq_off;
};

/* Userland helper ring state */
struct io_uring {
    int ring_fd;
    struct io_uring_params params;
    struct io_uring_sqe *sqes;
    struct io_uring_cqe *cqes;
    uint32_t sq_head;
    uint32_t sq_tail;
    uint32_t cq_head;
};

/* Syscall wrappers */
int io_uring_setup(uint32_t entries, struct io_uring_params *params);
int io_uring_enter(int fd, uint32_t to_submit, uint32_t min_complete, uint32_t flags);

/* Userland lifecycle helpers */
int io_uring_queue_init(uint32_t entries, struct io_uring *ring, uint32_t flags);
struct io_uring_sqe *io_uring_get_sqe(struct io_uring *ring);
int io_uring_submit(struct io_uring *ring);
int io_uring_peek_cqe(struct io_uring *ring, struct io_uring_cqe **cqe_ptr);
void io_uring_cqe_seen(struct io_uring *ring, struct io_uring_cqe *cqe);
void io_uring_queue_exit(struct io_uring *ring);

#endif /* _SYS_IO_URING_H */
