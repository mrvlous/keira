/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void print_num(int val) {
    if (val < 0) {
        print_char(45);
        val = -val;
    }
    if (val / 10 != 0) {
        print_num(val / 10);
    }
    print_char(48 + (val % 10));
}

void main(void) {
    printf("Keira Kernel Microbenchmark Suite (Ring 3 Userland)\n\n");

    /* 1. CPU Arithmetic Benchmark */
    int t0 = syscall(4, 0, 0, 0);
    int val = 1;
    int i = 0;
    while (i < 50000) {
        val = (val * 1664525 + 1013904223) / 7;
        i = i + 1;
    }
    int t1 = syscall(4, 0, 0, 0);
    printf("1. CPU Integer Arithmetic  : 50000 iterations in ");
    print_num(t1 - t0);
    printf(" ms\n");

    /* 2. Syscall Roundtrip Benchmark */
    int t2 = syscall(4, 0, 0, 0);
    int j = 0;
    while (j < 1000) {
        syscall(11, 0, 0, 0);
        j = j + 1;
    }
    int t3 = syscall(4, 0, 0, 0);
    printf("2. Syscall Latency         : 1000 getpid() calls in ");
    print_num(t3 - t2);
    printf(" ms\n");

    /* 3. Memory Allocation Benchmark */
    int heap_start = syscall(12, 0, 0, 0);
    int heap_end = syscall(12, heap_start + 4096, 0, 0);
    if (heap_end >= heap_start + 4096) {
        printf("3. Memory Allocation (Heap): Allocated 4096 bytes via brk [OK]\n");
    }

    printf("\n[DONE] Benchmark completed with zero faults.\n");
}
