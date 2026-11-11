// SPDX-License-Identifier: GPL-2.0-only

#include <syscall.h>
#include <unistd.h>

unsigned int sleep(unsigned int seconds) {
    sys_sleep(seconds * 1000);
    return 0;
}

int usleep(unsigned int usec) {
    sys_sleep(usec / 1000);
    return 0;
}
