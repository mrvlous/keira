/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdint.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <syscall.h>

#if defined(__x86_64__)
uintptr_t __stack_chk_guard = (uintptr_t)0x595e9fbd94fda766ULL;
#else
uintptr_t __stack_chk_guard = (uintptr_t)0x94fda766UL;
#endif

void __attribute__((noreturn)) __stack_chk_fail(void) {
    const char msg[] = "\n*** stack smashing detected ***: terminated\n";
    syscall3(SYS_WRITE, 2, (uint64_t)(uintptr_t)msg, sizeof(msg) - 1);
    exit(134);
    for (;;) {
    }
}
