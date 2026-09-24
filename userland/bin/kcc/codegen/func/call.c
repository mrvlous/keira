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
#include "symbols.h"

void emit_call(const char *name, int arg_count) {
#if defined(__i386__) || defined(__i686__)
    /* emit call rel32 */
    emit_u8(0xe8);
    int patch_pos = code_idx;
    emit_u32(0);

    if (arg_count > 0) {
        emit_u8(0x83);
        emit_u8(0xc4);
        emit_u8((unsigned char)(arg_count * 4));
    }

    if (patch_count < MAX_PATCHES) {
        k_strcpy(patch_names + patch_count * 32, name);
        patch_addresses[patch_count] = patch_pos;
        patch_count++;
    }
#else
    /* Pop stack arguments into calling registers in reverse order */
    if (arg_count >= 6)
        emit_pop_r9();
    if (arg_count >= 5)
        emit_pop_r8();
    if (arg_count >= 4)
        emit_pop_rcx();
    if (arg_count >= 3)
        emit_pop_rdx();
    if (arg_count >= 2)
        emit_pop_rsi();
    if (arg_count >= 1)
        emit_pop_rdi();

    /* emit call rel32 */
    emit_u8(0xe8);
    int patch_pos = code_idx;
    emit_u32(0);

    if (patch_count < MAX_PATCHES) {
        k_strcpy(patch_names + patch_count * 32, name);
        patch_addresses[patch_count] = patch_pos;
        patch_count++;
    }
#endif
}

void emit_syscall_trampoline(void) {
#if defined(__i386__) || defined(__i686__)
    /* Stack had args pushed: syscall nr, arg1, arg2, arg3 */
    emit_pop_rdx(); /* arg3 in edx */
    emit_pop_rcx(); /* arg2 in ecx */
    emit_u8(0x5b);  /* pop ebx (arg1) */
    emit_pop_rax(); /* syscall nr in eax */
    emit_u8(0xcd);
    emit_u8(0x80); /* int $0x80 */
#else
    /* Syscall convention: rax, rdi, rsi, rdx, r10, r8 */
    /* Stack had args pushed: syscall nr, arg1, arg2, arg3 */
    emit_pop_rdx();
    emit_pop_rsi();
    emit_pop_rdi();
    emit_pop_rax();
    emit_u8(0x0f);
    emit_u8(0x05); /* syscall */
#endif
}

void emit_printf_trampoline(int fmt_offset, int arg_count) {
#if defined(__i386__) || defined(__i686__)
    for (int a = 0; a < arg_count; a++) {
        emit_pop_rax();
    }

    /* Format string address in esi */
    emit_addr_global(fmt_offset);
    emit_u8(0x89);
    emit_u8(0xc6); /* mov esi, eax */

    int loop_top = code_idx;
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0x1e); /* movzx ebx, byte ptr [esi] */
    emit_u8(0x85);
    emit_u8(0xdb); /* test ebx, ebx */

    /* Emit raw jz rel32 (0x0f 0x84 imm32) */
    emit_u8(0x0f);
    emit_u8(0x84);
    int exit_jz = code_idx;
    emit_u32(0);

    /* sys_print_char (syscall nr = 1, arg1 = ebx) */
    emit_u8(0xb8);
    emit_u32(1);
    emit_u8(0xcd);
    emit_u8(0x80); /* int $0x80 */

    emit_u8(0xff);
    emit_u8(0xc6); /* inc esi */

    emit_jmp_backward(loop_top);
    patch_jump(exit_jz, code_idx);
#else
    /* If extra arguments were pushed on stack, pop them into rsi, rdx, rcx, r8,
     * r9 */
    if (arg_count >= 5)
        emit_pop_r9();
    if (arg_count >= 4)
        emit_pop_r8();
    if (arg_count >= 3)
        emit_pop_rcx();
    if (arg_count >= 2)
        emit_pop_rdx();
    if (arg_count >= 1)
        emit_pop_rsi();

    /* Format string address in rsi */
    emit_addr_global(fmt_offset);
    emit_u8(0x48);
    emit_u8(0x89);
    emit_u8(0xc6); /* mov rsi, rax */

    int loop_top = code_idx;
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0x3e); /* movzx edi, byte ptr [rsi] */
    emit_u8(0x85);
    emit_u8(0xff); /* test edi, edi */

    /* Emit raw jz rel32 (0x0f 0x84 imm32) */
    emit_u8(0x0f);
    emit_u8(0x84);
    int exit_jz = code_idx;
    emit_u32(0);

    /* sys_print_char (syscall nr = 1, arg1 = edi) */
    emit_u8(0xb8);
    emit_u32(1);
    emit_u8(0x0f);
    emit_u8(0x05); /* syscall */

    emit_u8(0x48);
    emit_u8(0xff);
    emit_u8(0xc6); /* inc rsi */

    emit_jmp_backward(loop_top);
    patch_jump(exit_jz, code_idx);
#endif
}
