/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <sched.h>
#include <stdint.h>
#include <sys/syscall.h>

#if defined(__x86_64__)

__attribute__((naked)) int clone(int (*fn)(void *), void *child_stack, int flags, void *arg,
                                 int *ptid, void *newtls, int *ctid) {
    __asm__ volatile("test %rdi, %rdi\n\t"
                     "jz .L_einval\n\t"
                     "test %rsi, %rsi\n\t"
                     "jz .L_einval\n\t"

                     "mov 8(%rsp), %r11\n\t"
                     "and $-16, %rsi\n\t"
                     "sub $16, %rsi\n\t"
                     "mov %rcx, (%rsi)\n\t"
                     "mov %rdi, 8(%rsi)\n\t"

                     "mov %rdx, %rdi\n\t"
                     "mov %r8, %rdx\n\t"
                     "mov %r11, %r10\n\t"
                     "mov %r9, %r8\n\t"
                     "mov $41, %eax\n\t"

                     "syscall\n\t"

                     "test %rax, %rax\n\t"
                     "jnz .L_parent_ret\n\t"

                     "xor %ebp, %ebp\n\t"
                     "pop %rdi\n\t"
                     "pop %rax\n\t"
                     "call *%rax\n\t"

                     "mov %rax, %rdi\n\t"
                     "mov $2, %eax\n\t"
                     "syscall\n\t"

                     ".L_child_hang:\n\t"
                     "pause\n\t"
                     "jmp .L_child_hang\n\t"

                     ".L_parent_ret:\n\t"
                     "cmp $0, %rax\n\t"
                     "jge .L_done\n\t"
                     "neg %rax\n\t"
                     "mov %eax, errno(%rip)\n\t"
                     "mov $-1, %rax\n\t"
                     ".L_done:\n\t"
                     "ret\n\t"

                     ".L_einval:\n\t"
                     "movl $22, errno(%rip)\n\t"
                     "mov $-1, %rax\n\t"
                     "ret\n\t");
}

#elif defined(__i386__)

__attribute__((naked)) int clone(int (*fn)(void *), void *child_stack, int flags, void *arg,
                                 int *ptid, void *newtls, int *ctid) {
    __asm__ volatile("push %ebx\n\t"
                     "push %esi\n\t"
                     "push %edi\n\t"
                     "push %ebp\n\t"

                     "mov 20(%esp), %eax\n\t"
                     "mov 24(%esp), %ecx\n\t"
                     "test %eax, %eax\n\t"
                     "jz .L_einval32\n\t"
                     "test %ecx, %ecx\n\t"
                     "jz .L_einval32\n\t"

                     "and $-16, %ecx\n\t"
                     "sub $16, %ecx\n\t"
                     "mov 32(%esp), %edx\n\t"
                     "mov %edx, (%ecx)\n\t"
                     "mov %eax, 4(%ecx)\n\t"

                     "mov 28(%esp), %ebx\n\t"
                     "mov 36(%esp), %edx\n\t"
                     "mov 44(%esp), %esi\n\t"
                     "mov 40(%esp), %edi\n\t"
                     "mov $41, %eax\n\t"

                     "int $0x80\n\t"

                     "test %eax, %eax\n\t"
                     "jnz .L_parent_ret32\n\t"

                     "xor %ebp, %ebp\n\t"
                     "pop %eax\n\t"
                     "pop %edx\n\t"
                     "push %eax\n\t"
                     "call *%edx\n\t"
                     "push %eax\n\t"
                     "mov $2, %eax\n\t"
                     "int $0x80\n\t"

                     ".L_child_hang32:\n\t"
                     "pause\n\t"
                     "jmp .L_child_hang32\n\t"

                     ".L_parent_ret32:\n\t"
                     "pop %ebp\n\t"
                     "pop %edi\n\t"
                     "pop %esi\n\t"
                     "pop %ebx\n\t"
                     "cmp $0, %eax\n\t"
                     "jge .L_done32\n\t"
                     "neg %eax\n\t"
                     "mov %eax, errno\n\t"
                     "mov $-1, %eax\n\t"
                     ".L_done32:\n\t"
                     "ret\n\t"

                     ".L_einval32:\n\t"
                     "pop %ebp\n\t"
                     "pop %edi\n\t"
                     "pop %esi\n\t"
                     "pop %ebx\n\t"
                     "movl $22, errno\n\t"
                     "mov $-1, %eax\n\t"
                     "ret\n\t");
}

#endif
