// SPDX-License-Identifier: GPL-2.0-only

#include <stdlib.h>

static unsigned int rand_seed = 1;

int rand(void) {
    rand_seed = rand_seed * 1103515245 + 12345;
    return (unsigned int)(rand_seed / 65536) % 32768;
}

void srand(unsigned int seed) {
    rand_seed = seed;
}
