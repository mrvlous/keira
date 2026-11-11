// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <stdint.h>
#include <sys/reboot.h>
#include <sys/syscall.h>
#include <unistd.h>

int sys_reboot(int magic1, int magic2, int cmd, void *arg) {
    int64_t ret = syscall4(SYS_REBOOT, (uint64_t)(uint32_t)magic1, (uint64_t)(uint32_t)magic2,
                           (uint64_t)(uint32_t)cmd, (uint64_t)(uintptr_t)arg);
    return (int)ret;
}

int reboot(int cmd) {
    int ret = sys_reboot((int)LINUX_REBOOT_MAGIC1, (int)LINUX_REBOOT_MAGIC2, cmd, NULL);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
