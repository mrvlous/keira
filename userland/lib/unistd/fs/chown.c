// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <sys/syscall.h>
#include <unistd.h>

int chown(const char *pathname, uid_t owner, gid_t group) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    int ret = sys_chown(pathname, owner, group);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
