/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "fuzz_abi.h"

#include <stdint.h>
#include <syscall.h>
#include <unistd.h>

static const uint64_t BOUNDARY_VALS[] = {
    0,
    1,
    (uint64_t)-1,
    0x7FFFFFFF,
    0x80000000,
    0xFFFFFFFF,
    (uint64_t)0xDEADBEEF,
    KERNEL_SPACE_PTR,
    KERNEL_SPACE_PTR + 0x1000,
    (uint64_t)0x00007FFFFFFFFFFFULL,
    (uint64_t)0xFFFFFFFFFFFFFFFFULL,
    0x1001,
    0x4000,
    0x10000,
};
#define NUM_BOUNDARIES (sizeof(BOUNDARY_VALS) / sizeof(BOUNDARY_VALS[0]))

static uint64_t g_rng_seed = 0x853c49e6748fea9bULL;

void init_fuzzer_prng(void) {
    time_t start_time = sys_uptime();
    g_rng_seed ^= (uint64_t)start_time ^ ((uint64_t)getpid() << 32);
}

uint64_t xorshift64(void) {
    uint64_t x = g_rng_seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    g_rng_seed = x;
    return x;
}

uint64_t get_mutated_arg(void) {
    uint64_t r = xorshift64();
    uint32_t choice = (uint32_t)(r % 10);
    if (choice < 4) {
        return BOUNDARY_VALS[r % NUM_BOUNDARIES];
    } else if (choice < 6) {
        return r % 64;
    } else if (choice < 8) {
        static char dummy_buf[256];
        return (uint64_t)(uintptr_t)dummy_buf + (r % 128);
    } else {
        return r;
    }
}
