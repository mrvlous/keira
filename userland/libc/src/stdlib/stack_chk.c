// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
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
