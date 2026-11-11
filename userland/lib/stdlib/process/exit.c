// SPDX-License-Identifier: GPL-2.0-only

#include <stdio.h>
#include <stdlib.h>
#include <syscall.h>

void exit(int status) {
    if (stdout) {
        fflush(stdout);
    }
    if (stderr) {
        fflush(stderr);
    }
    sys_exit(status);
}

void abort(void) {
    sys_exit(134);
}
