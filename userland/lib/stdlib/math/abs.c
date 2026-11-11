// SPDX-License-Identifier: GPL-2.0-only

#include <stdlib.h>

long long llabs(long long j) {
    if (j < 0)
        return -j;
    return j;
}
