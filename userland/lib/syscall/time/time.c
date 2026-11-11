// SPDX-License-Identifier: GPL-2.0-only

#include <stdint.h>
#include <sys/syscall.h>
#include <time.h>

void sys_sleep(uint32_t ms) {
    syscall1(SYS_SLEEP, (uint64_t)ms);
}

time_t sys_uptime(void) {
    return (time_t)syscall0(SYS_UPTIME);
}
