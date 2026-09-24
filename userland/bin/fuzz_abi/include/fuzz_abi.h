/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _FUZZ_ABI_H
#define _FUZZ_ABI_H

#include <stdint.h>
#include <sys/types.h>

#define TOTAL_FUZZ_ITERATIONS 10000

#if defined(__x86_64__)
#define KERNEL_SPACE_PTR ((uint64_t)0xFFFF800000000000ULL)
#else
#define KERNEL_SPACE_PTR ((uint64_t)0xC0000000UL)
#endif

void init_fuzzer_prng(void);
uint64_t xorshift64(void);
uint64_t get_mutated_arg(void);

int run_phase1_syscall_fuzzing(int *out_calls);
int run_phase2_fd_exhaustion(int *out_opened);
int run_phase3_memory_churn(void);
int run_phase4_process_churn(int *out_churn);
int run_phase5_signal_storm(int *out_bursts);

#endif /* _FUZZ_ABI_H */
