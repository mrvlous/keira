// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <sys/syscall.h>
#include <sys/utsname.h>
#include <unistd.h>

int sys_uname(void *buf) {
    int64_t ret = syscall1(SYS_UNAME, (uint64_t)(uintptr_t)buf);
    return (int)ret;
}

int uname(struct utsname *buf) {
    int ret = sys_uname((void *)buf);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
