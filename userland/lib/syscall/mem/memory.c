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
#include <sys/mman.h>
#include <sys/syscall.h>

void *sys_mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    int64_t ret =
        syscall6(SYS_MMAP, (uint64_t)(uintptr_t)addr, (uint64_t)length, (uint64_t)(uint32_t)prot,
                 (uint64_t)(uint32_t)flags, (uint64_t)(int64_t)fd, (uint64_t)offset);
    return (void *)(uintptr_t)ret;
}

int sys_munmap(void *addr, size_t length) {
    return (int)syscall2(SYS_MUNMAP, (uint64_t)(uintptr_t)addr, (uint64_t)length);
}

int sys_mprotect(void *addr, size_t len, int prot) {
    return (int)syscall3(SYS_MPROTECT, (uint64_t)(uintptr_t)addr, (uint64_t)len,
                         (uint64_t)(uint32_t)prot);
}

int sys_msync(void *addr, size_t length, int flags) {
    return (int)syscall3(SYS_MSYNC, (uint64_t)(uintptr_t)addr, (uint64_t)length,
                         (uint64_t)(uint32_t)flags);
}

void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return sys_mmap(addr, length, prot, flags, fd, offset);
}

int munmap(void *addr, size_t length) {
    return sys_munmap(addr, length);
}

int mprotect(void *addr, size_t len, int prot) {
    return sys_mprotect(addr, len, prot);
}

int msync(void *addr, size_t length, int flags) {
    return sys_msync(addr, length, flags);
}
