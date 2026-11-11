// SPDX-License-Identifier: GPL-2.0-only

#include <stdint.h>
#include <sys/syscall.h>

int sys_ioctl(int fd, unsigned long request, void *argp) {
    return (int)syscall3(SYS_IOCTL, (uint64_t)fd, (uint64_t)request, (uint64_t)(uintptr_t)argp);
}
