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
#include <sys/syscall.h>

#if defined(__i386__) || defined(__i686__)

int64_t syscall0(uint64_t num) {
    int32_t ret;
    __asm__ volatile("int $0x80" : "=a"(ret) : "a"((uint32_t)num) : "memory");
    return (int64_t)ret;
}

int64_t syscall1(uint64_t num, uint64_t a1) {
    int32_t ret;
    __asm__ volatile("int $0x80" : "=a"(ret) : "a"((uint32_t)num), "b"((uint32_t)a1) : "memory");
    return (int64_t)ret;
}

int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4), "D"((uint32_t)a5)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5,
                 uint64_t a6) {
    int32_t ret;
    uint32_t arg6 = (uint32_t)a6;
    __asm__ volatile("pushl %%ebp\n\t"
                     "movl %7, %%ebp\n\t"
                     "int $0x80\n\t"
                     "popl %%ebp\n\t"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4), "D"((uint32_t)a5), "m"(arg6)
                     : "memory");
    return (int64_t)ret;
}

#else

int64_t syscall0(uint64_t num) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall1(uint64_t num, uint64_t a1) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num), "D"(a1) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num), "D"(a1), "S"(a2) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3) {
    int64_t ret;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    register uint64_t r8 __asm__("r8") = a5;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10), "r"(r8)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5,
                 uint64_t a6) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    register uint64_t r8 __asm__("r8") = a5;
    register uint64_t r9 __asm__("r9") = a6;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10), "r"(r8), "r"(r9)
                     : "rcx", "r11", "memory");
    return ret;
}

#endif
