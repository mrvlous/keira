/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "codegen.h"

#include "lexer.h"
#include "symbols.h"

#if defined(__i386__) || defined(__i686__)
#define REX_W()
#else
#define REX_W() emit_u8(0x48)
#endif

/* Byte emission primitives */
void emit_u8(unsigned char val) {
    if (code_idx >= MAX_CODE_SIZE) {
        error_msg("Code segment overflow");
        return;
    }
    code_buf[code_idx++] = val;
}

void emit_u16(unsigned short val) {
    emit_u8((unsigned char)(val & 0xFF));
    emit_u8((unsigned char)((val >> 8) & 0xFF));
}

void emit_u32(unsigned int val) {
    emit_u8((unsigned char)(val & 0xFF));
    emit_u8((unsigned char)((val >> 8) & 0xFF));
    emit_u8((unsigned char)((val >> 16) & 0xFF));
    emit_u8((unsigned char)((val >> 24) & 0xFF));
}

void emit_u64(uint64_t val) {
    emit_u32((unsigned int)(val & 0xFFFFFFFF));
    emit_u32((unsigned int)((val >> 32) & 0xFFFFFFFF));
}

/* Register & Immediate Operations */
void emit_load_imm(long val) {
#if defined(__i386__) || defined(__i686__)
    /* mov eax, imm32 */
    emit_u8(0xb8);
    emit_u32((unsigned int)val);
#else
    /* mov rax, imm64 */
    emit_u8(0x48);
    emit_u8(0xb8);
    emit_u64((unsigned long)val);
#endif
}

void emit_push_rax(void) {
    emit_u8(0x50); /* push rax / eax */
}

void emit_pop_rax(void) {
    emit_u8(0x58); /* pop rax / eax */
}

void emit_pop_rcx(void) {
    emit_u8(0x59); /* pop rcx / ecx */
}

void emit_pop_rdx(void) {
    emit_u8(0x5a); /* pop rdx / edx */
}

void emit_pop_rsi(void) {
    emit_u8(0x5e); /* pop rsi / esi */
}

void emit_pop_rdi(void) {
    emit_u8(0x5f); /* pop rdi / edi */
}

void emit_pop_r8(void) {
    emit_u8(0x41);
    emit_u8(0x58); /* pop r8 */
}

void emit_pop_r9(void) {
    emit_u8(0x41);
    emit_u8(0x59); /* pop r9 */
}

/* Binary Arithmetic (rcx = LHS, rax = RHS) */
void emit_add(void) {
    /* add rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x01);
    emit_u8(0xc8);
}

void emit_sub(void) {
    /* sub rcx/ecx, rax/eax; mov rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x29);
    emit_u8(0xc1);
    REX_W();
    emit_u8(0x89);
    emit_u8(0xc8);
}

void emit_imul(void) {
    /* imul rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xaf);
    emit_u8(0xc1);
}

void emit_idiv(void) {
    /* xchg rax, rcx; cqo/cdq; idiv rcx */
    REX_W();
    emit_u8(0x91); /* xchg rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x99); /* cqo / cdq */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xf9); /* idiv rcx/ecx */
}

void emit_imod(void) {
    /* xchg rax, rcx; cqo/cdq; idiv rcx; mov rax, rdx */
    REX_W();
    emit_u8(0x91); /* xchg rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x99); /* cqo / cdq */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xf9); /* idiv rcx/ecx */
    REX_W();
    emit_u8(0x89);
    emit_u8(0xd0); /* mov rax/eax, rdx/edx */
}

/* Bitwise Operations (rcx = LHS, rax = RHS) */
void emit_bit_and(void) {
    /* and rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x21);
    emit_u8(0xc8);
}

void emit_bit_or(void) {
    /* or rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x09);
    emit_u8(0xc8);
}

void emit_bit_xor(void) {
    /* xor rax/eax, rcx/ecx */
    REX_W();
    emit_u8(0x31);
    emit_u8(0xc8);
}

void emit_bit_not(void) {
    /* not rax/eax */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xd0);
}

void emit_shl(void) {
    /* xchg rax/eax, rcx/ecx; shl rax/eax, cl */
    REX_W();
    emit_u8(0x91); /* xchg */
    REX_W();
    emit_u8(0xd3);
    emit_u8(0xe0); /* shl */
}

void emit_shr(void) {
    /* xchg rax/eax, rcx/ecx; sar rax/eax, cl */
    REX_W();
    emit_u8(0x91); /* xchg */
    REX_W();
    emit_u8(0xd3);
    emit_u8(0xf8); /* sar */
}

/* Logical & Unary Operations */
void emit_log_and(void) {
    /* test rcx, rcx; setne cl; test rax, rax; setne al; and al, cl; movzx rax, al */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc9); /* test rcx/ecx, rcx/ecx */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc1); /* setne cl */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0); /* test rax/eax, rax/eax */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc0); /* setne al */
    emit_u8(0x20);
    emit_u8(0xc8); /* and al, cl */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}

void emit_log_or(void) {
    /* or rcx, rax; test rcx, rcx; setne al; movzx rax, al */
    REX_W();
    emit_u8(0x09);
    emit_u8(0xc1); /* or rcx/ecx, rax/eax */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc9); /* test rcx/ecx, rcx/ecx */
    emit_u8(0x0f);
    emit_u8(0x95);
    emit_u8(0xc0); /* setne al */
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}

void emit_log_not(void) {
    /* test rax, rax; sete al; movzx rax, al */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x94);
    emit_u8(0xc0);
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0);
}

void emit_neg(void) {
    /* neg rax/eax */
    REX_W();
    emit_u8(0xf7);
    emit_u8(0xd8);
}

/* Comparison and Relational Sets (rcx = LHS, rax = RHS) */
void emit_cmp_set(int op_tok) {
    /* cmp rcx/ecx, rax/eax */
    REX_W();
    emit_u8(0x39);
    emit_u8(0xc1);

    switch (op_tok) {
    case TOK_EQ:
        emit_u8(0x0f);
        emit_u8(0x94);
        emit_u8(0xc0); /* sete al */
        break;
    case TOK_NEQ:
        emit_u8(0x0f);
        emit_u8(0x95);
        emit_u8(0xc0); /* setne al */
        break;
    case TOK_LT:
        emit_u8(0x0f);
        emit_u8(0x9c);
        emit_u8(0xc0); /* setl al */
        break;
    case TOK_GT:
        emit_u8(0x0f);
        emit_u8(0x9f);
        emit_u8(0xc0); /* setg al */
        break;
    case TOK_LEQ:
        emit_u8(0x0f);
        emit_u8(0x9e);
        emit_u8(0xc0); /* setle al */
        break;
    case TOK_GEQ:
        emit_u8(0x0f);
        emit_u8(0x9d);
        emit_u8(0xc0); /* setge al */
        break;
    default:
        break;
    }
    REX_W();
    emit_u8(0x0f);
    emit_u8(0xb6);
    emit_u8(0xc0); /* movzx rax/eax, al */
}

/* Local Variables & Stack Frame Handling */
void emit_load_local(int offset, int size) {
    if (size == 1) {
        /* movzx rax/eax, byte ptr [rbp/ebp + offset] */
        REX_W();
        emit_u8(0x0f);
        emit_u8(0xb6);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    } else {
        /* mov rax/eax, [rbp/ebp + offset] */
        REX_W();
        emit_u8(0x8b);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    }
}

void emit_store_local(int offset, int size) {
    if (size == 1) {
        /* mov byte ptr [rbp/ebp + offset], al */
        emit_u8(0x88);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    } else {
        /* mov [rbp/ebp + offset], rax/eax */
        REX_W();
        emit_u8(0x89);
        if (offset >= -128 && offset <= 127) {
            emit_u8(0x45);
            emit_u8((unsigned char)offset);
        } else {
            emit_u8(0x85);
            emit_u32((unsigned int)offset);
        }
    }
}

void emit_addr_local(int offset) {
    /* lea rax/eax, [rbp/ebp + offset] */
    REX_W();
    emit_u8(0x8d);
    if (offset >= -128 && offset <= 127) {
        emit_u8(0x45);
        emit_u8((unsigned char)offset);
    } else {
        emit_u8(0x85);
        emit_u32((unsigned int)offset);
    }
}

void emit_inc_local(int offset, int is_post, int is_dec) {
    emit_load_local(offset, 8);
    if (is_post) {
        emit_push_rax(); /* save original value for expression result */
    }
    if (is_dec) {
        REX_W();
        emit_u8(0xff);
        emit_u8(0xc8); /* dec rax/eax */
    } else {
        REX_W();
        emit_u8(0xff);
        emit_u8(0xc0); /* inc rax/eax */
    }
    emit_store_local(offset, 8);
    if (is_post) {
        emit_pop_rax(); /* return original */
    }
}

/* Global Variables & Data Segment Addressing */
void emit_addr_global(int offset) {
#if defined(__i386__) || defined(__i686__)
    emit_u8(0xb8);
    int patch_pos = code_idx;
    emit_u32((unsigned int)offset);
#else
    emit_u8(0x48);
    emit_u8(0xb8);
    int patch_pos = code_idx;
    emit_u64((unsigned long)offset);
#endif

    if (val_patch_count < MAX_VAL_PATCHES) {
        val_patch_addresses[val_patch_count] = patch_pos;
        val_patch_offsets[val_patch_count] = offset;
        val_patch_count++;
    }
}

void emit_load_global(int offset, int size) {
    emit_addr_global(offset);
    emit_deref(size);
}

void emit_store_global(int offset, int size) {
    /* rax/eax has value to store */
    emit_push_rax();
    emit_addr_global(offset);
    /* rax/eax = address, top of stack = value */
    REX_W();
    emit_u8(0x89);
    emit_u8(0xc2);  /* mov rdx/edx, rax/eax (address) */
    emit_pop_rax(); /* rax/eax = value */
    emit_store_deref(size);
}

/* Pointer Dereferencing */
void emit_deref(int size) {
    if (size == 1) {
        /* movzx rax/eax, byte ptr [rax/eax] */
        REX_W();
        emit_u8(0x0f);
        emit_u8(0xb6);
        emit_u8(0x00);
    } else {
        /* mov rax/eax, [rax/eax] */
        REX_W();
        emit_u8(0x8b);
        emit_u8(0x00);
    }
}

void emit_store_deref(int size) {
    /* rdx/edx has pointer address, rax/eax has value to write */
    if (size == 1) {
        /* mov byte ptr [rdx/edx], al */
        emit_u8(0x88);
        emit_u8(0x02);
    } else {
        /* mov [rdx/edx], rax/eax */
        REX_W();
        emit_u8(0x89);
        emit_u8(0x02);
    }
}

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

void emit_syscall_stub(void) {
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

void emit_printf_stub(int fmt_offset, int arg_count) {
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
    /* If extra arguments were pushed on stack, pop them into rsi, rdx, rcx, r8, r9 */
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

/* Control Flow, Branches & Jump Patching */
int emit_jmp_forward(void) {
    emit_u8(0xe9);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

void emit_jmp_backward(int target_addr) {
    emit_u8(0xe9);
    int rel = target_addr - (code_idx + 4);
    emit_u32((unsigned int)rel);
}

int emit_jz_forward(void) {
    /* test rax/eax, rax/eax; jz rel32 */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x84);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

int emit_jnz_forward(void) {
    /* test rax/eax, rax/eax; jnz rel32 */
    REX_W();
    emit_u8(0x85);
    emit_u8(0xc0);
    emit_u8(0x0f);
    emit_u8(0x85);
    int patch_pos = code_idx;
    emit_u32(0);
    return patch_pos;
}

void patch_jump(int patch_pos, int target_pos) {
    int rel_offset = target_pos - (patch_pos + 4);
    k_memcpy((char *)(code_buf + patch_pos), (char *)&rel_offset, 4);
}
