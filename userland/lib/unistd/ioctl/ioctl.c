// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <stdarg.h>
#include <stdint.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>

int ioctl(int fd, unsigned long request, ...) {
    va_list ap;
    va_start(ap, request);
    // NOLINTNEXTLINE(clang-analyzer-optin.core.UnconditionalVAArg)
    void *argp = va_arg(ap, void *);
    va_end(ap);

    int ret = (int)syscall3(SYS_IOCTL, (uint64_t)fd, (uint64_t)request, (uint64_t)(uintptr_t)argp);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}
