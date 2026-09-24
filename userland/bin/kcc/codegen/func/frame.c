/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "codegen.h"

/* Function Prologue, Epilogue & Calling Conventions */
void emit_func_prologue(void) {
    /* push rbp/ebp; mov rbp/ebp, rsp/esp; sub rsp/esp, 1024 */
    emit_u8(0x55);
    REX_W();
    emit_u8(0x89);
    emit_u8(0xe5);
    REX_W();
    emit_u8(0x81);
    emit_u8(0xec);
    emit_u32(1024);
}

void emit_func_epilogue(int is_main) {
    /* mov rsp/esp, rbp/ebp; pop rbp/ebp */
    REX_W();
    emit_u8(0x89);
    emit_u8(0xec);
    emit_u8(0x5d);

    if (is_main) {
#if defined(__i386__) || defined(__i686__)
        /* sys_exit(eax): mov ebx, eax; mov eax, 2; int $0x80; jmp $ */
        emit_u8(0x89);
        emit_u8(0xc3); /* mov ebx, eax */
        emit_u8(0xb8);
        emit_u32(2); /* sys_exit syscall nr = 2 */
        emit_u8(0xcd);
        emit_u8(0x80); /* int $0x80 */
        emit_u8(0xeb);
        emit_u8(0xfe); /* jmp $ */
#else
        /* sys_exit(rax): mov rdi, rax; mov rax, 2; syscall; jmp $ */
        emit_u8(0x48);
        emit_u8(0x89);
        emit_u8(0xc7); /* mov rdi, rax */
        emit_u8(0xb8);
        emit_u32(2); /* sys_exit syscall nr = 2 */
        emit_u8(0x0f);
        emit_u8(0x05); /* syscall */
        emit_u8(0xeb);
        emit_u8(0xfe); /* jmp $ */
#endif
    } else {
        emit_u8(0xc3); /* ret */
    }
}

void emit_param_save(int param_idx, int local_offset) {
#if defined(__i386__) || defined(__i686__)
    int caller_arg_offset = 8 + param_idx * 4;
    emit_u8(0x8b);
    emit_u8(0x45);
    emit_u8((unsigned char)caller_arg_offset);
    emit_u8(0x89);
    emit_u8(0x45);
    emit_u8((unsigned char)local_offset);
#else
    /* Store incoming register into [rbp + local_offset] */
    switch (param_idx) {
    case 0: /* rdi */
        emit_u8(0x48);
        emit_u8(0x89);
        emit_u8(0x7d);
        emit_u8((unsigned char)local_offset);
        break;
    case 1: /* rsi */
        emit_u8(0x48);
        emit_u8(0x89);
        emit_u8(0x75);
        emit_u8((unsigned char)local_offset);
        break;
    case 2: /* rdx */
        emit_u8(0x48);
        emit_u8(0x89);
        emit_u8(0x55);
        emit_u8((unsigned char)local_offset);
        break;
    case 3: /* rcx */
        emit_u8(0x48);
        emit_u8(0x89);
        emit_u8(0x4d);
        emit_u8((unsigned char)local_offset);
        break;
    case 4: /* r8 */
        emit_u8(0x4c);
        emit_u8(0x89);
        emit_u8(0x45);
        emit_u8((unsigned char)local_offset);
        break;
    case 5: /* r9 */
        emit_u8(0x4c);
        emit_u8(0x89);
        emit_u8(0x4d);
        emit_u8((unsigned char)local_offset);
        break;
    default:
        break;
    }
#endif
}
