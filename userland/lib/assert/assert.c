// SPDX-License-Identifier: GPL-2.0-only

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

void __assert_fail(const char *assertion, const char *file, unsigned int line,
                   const char *function) {
    const char *fn = function;
    if (!fn) {
        fn = "unknown";
    }
    printf("[ASSERTION FAILED] %s in %s (%s:%u)\n", assertion, fn, file, line);
    abort();
}
