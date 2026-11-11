// SPDX-License-Identifier: GPL-2.0-only

#include <setjmp.h>

int setjmp(jmp_buf env) {
    (void)env;
    return 0;
}

void longjmp(jmp_buf env, int val) {
    (void)env;
    (void)val;
}
