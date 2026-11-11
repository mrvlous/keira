// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <stdint.h>
#include <sys/syscall.h>
#include <syscall.h>
#include <unistd.h>

int pipe(int pipefd[2]) {
    if (!pipefd) {
        errno = EFAULT;
        return -1;
    }
    int64_t ret = syscall1(SYS_PIPE, (uint64_t)(uintptr_t)pipefd);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

int pipe2(int pipefd[2], int flags) {
    if (!pipefd) {
        errno = EFAULT;
        return -1;
    }
    int64_t ret = syscall2(SYS_PIPE2, (uint64_t)(uintptr_t)pipefd, (uint64_t)flags);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}
