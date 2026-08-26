/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _KCC_CODEGEN_H
#define _KCC_CODEGEN_H

#include "common.h"

/* Byte emission primitives */
void emit_u8(unsigned char val);
void emit_u16(unsigned short val);
void emit_u32(unsigned int val);
void emit_u64(uint64_t val);

/* Register & Immediate Operations */
void emit_load_imm(long val);
void emit_push_rax(void);
void emit_pop_rax(void);
void emit_pop_rcx(void);
void emit_pop_rdx(void);
void emit_pop_rsi(void);
void emit_pop_rdi(void);
void emit_pop_r8(void);
void emit_pop_r9(void);

/* Binary Arithmetic & Modulo */
void emit_add(void);
void emit_sub(void);
void emit_imul(void);
void emit_idiv(void);
void emit_imod(void);

/* Bitwise & Shifts */
void emit_bit_and(void);
void emit_bit_or(void);
void emit_bit_xor(void);
void emit_bit_not(void);
void emit_shl(void);
void emit_shr(void);

/* Logical & Unary Operations */
void emit_log_and(void);
void emit_log_or(void);
void emit_log_not(void);
void emit_neg(void);

/* Comparison & Relational Sets */
void emit_cmp_set(int op_tok);

/* Local Variables & Stack Frames */
void emit_load_local(int offset, int size);
void emit_store_local(int offset, int size);
void emit_addr_local(int offset);
void emit_inc_local(int offset, int is_post, int is_dec);

/* Global Variables & Data Segment Addressing */
void emit_load_global(int offset, int size);
void emit_store_global(int offset, int size);
void emit_addr_global(int offset);

/* Pointer Dereferencing */
void emit_deref(int size);
void emit_store_deref(int size);

/* Function Prologue, Epilogue & System V AMD64 ABI Calls */
void emit_func_prologue(void);
void emit_func_epilogue(int is_main);
void emit_param_save(int param_idx, int local_offset);
void emit_call(const char *name, int arg_count);
void emit_syscall_stub(void);
void emit_printf_stub(int fmt_offset, int arg_count);

/* Control Flow, Branches & Jump Patching */
int emit_jmp_forward(void);
void emit_jmp_backward(int target_addr);
int emit_jz_forward(void);
int emit_jnz_forward(void);
void patch_jump(int patch_pos, int target_pos);

#endif /* _KCC_CODEGEN_H */
