// SPDX-License-Identifier: GPL-2.0-only

#include <stdint.h>
#include <sys/io_uring.h>
#include <sys/syscall.h>

int io_uring_setup(uint32_t entries, struct io_uring_params *params) {
    return (int)syscall2(SYS_IO_URING_SETUP, (uint64_t)entries, (uint64_t)(uintptr_t)params);
}

int io_uring_enter(int fd, uint32_t to_submit, uint32_t min_complete, uint32_t flags) {
    return (int)syscall4(SYS_IO_URING_ENTER, (uint64_t)fd, (uint64_t)to_submit,
                         (uint64_t)min_complete, (uint64_t)flags);
}
