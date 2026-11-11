// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <stdint.h>
#include <sys/syscall.h>
#include <syscall.h>
#include <unistd.h>

int brk(void *addr) {
    int64_t ret = syscall1(SYS_BRK, (uint64_t)(uintptr_t)addr);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

void *sbrk(intptr_t increment) {
    int64_t ret = syscall1(SYS_BRK, (uint64_t)increment);
    if (ret < 0) {
        errno = (int)-ret;
        return (void *)-1;
    }
    return (void *)(uintptr_t)ret;
}
