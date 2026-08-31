// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Rust handler dispatcher for CPU exceptions and userland crash recovery.

use keira_arch::debug::unwind::unwind_from_frame;
use keira_io::serial;
use keira_io::vga;
use keira_task::scheduler::{exit_current, CURRENT_TASK_IDX};

#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
pub struct ExceptionStackFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
    pub vector: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[cfg(target_arch = "x86")]
#[repr(C, packed)]
pub struct ExceptionStackFrame {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp_dummy: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub vector: u32,
    pub error_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub user_esp: u32,
    pub user_ss: u32,
}

/// CPU exception dispatcher invoked by low-level assembly ISR stubs.
#[no_mangle]
pub unsafe extern "C" fn exception_dispatcher(frame_ptr: *const ExceptionStackFrame) {
    let frame = &*frame_ptr;

    #[cfg(target_arch = "x86_64")]
    let (vector, error_code, rip, rsp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi, rdi, rbp) = (
        frame.vector,
        frame.error_code,
        frame.rip,
        frame.rsp,
        frame.cs,
        frame.ss,
        frame.rflags,
        frame.rax,
        frame.rbx,
        frame.rcx,
        frame.rdx,
        frame.rsi,
        frame.rdi,
        frame.rbp,
    );

    #[cfg(target_arch = "x86")]
    let (vector, error_code, rip, rsp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi, rdi, rbp) = (
        frame.vector as u64,
        frame.error_code as u64,
        frame.eip as u64,
        frame.user_esp as u64,
        frame.cs as u64,
        frame.user_ss as u64,
        frame.eflags as u64,
        frame.eax as u64,
        frame.ebx as u64,
        frame.ecx as u64,
        frame.edx as u64,
        frame.esi as u64,
        frame.edi as u64,
        frame.ebp as u64,
    );

    if (cs & 3) == 3 {
        // 1. Attempt to resolve user mode Page Fault on-demand (Demand Paging / Stack Auto-Growth)
        if vector == 14 {
            let cr2 = unsafe { keira_arch::cpu::read_cr2() } as u64;
            if unsafe { keira_mem::vmm::handle_page_fault(cr2, error_code, rsp) } {
                return;
            }
        }

        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("\n*** USER PROCESS CRASHED ***\n");
        vga::print_str("Exception Vector: ");
        vga::print_u64(vector);
        if vector == 14 {
            vga::print_str(" (Page Fault)");
        } else if vector == 13 {
            vga::print_str(" (General Protection Fault)");
        }
        vga::print_str("\nRIP: 0x");
        print_hex(rip);
        if vector == 14 {
            let cr2 = unsafe { keira_arch::cpu::read_cr2() } as u64;
            vga::print_str(" | Faulting Address: 0x");
            print_hex(cr2);

            keira_io::serial::print_str("\n[PAGE FAULT] RIP: 0x");
            keira_io::serial::print_u64(rip);
            keira_io::serial::print_str(" | CR2: 0x");
            keira_io::serial::print_u64(cr2);
            keira_io::serial::print_str(" | RSP: 0x");
            keira_io::serial::print_u64(rsp);
            keira_io::serial::print_str(" | Error Code: 0x");
            keira_io::serial::print_u64(error_code);
            keira_io::serial::print_str("\n");
        }
        vga::print_str("\nTerminating crashed user process...\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        if CURRENT_TASK_IDX != 0 {
            exit_current(-11);
        } else {
            extern "C" {
                fn abort_user_mode() -> !;
            }
            abort_user_mode();
        }
    }

    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("\n*** KERNEL PANIC ***\n");
    vga::print_str("UNHANDLED CPU EXCEPTION: ");
    match vector {
        0 => vga::print_str("Division by Zero (#DE)"),
        1 => vga::print_str("Debug Exception (#DB)"),
        2 => vga::print_str("Non-Maskable Interrupt (NMI)"),
        3 => vga::print_str("Breakpoint (#BP)"),
        4 => vga::print_str("Overflow (#OF)"),
        5 => vga::print_str("Bound Range Exceeded (#BR)"),
        6 => vga::print_str("Invalid Opcode (#UD)"),
        7 => vga::print_str("Device Not Available (#NM)"),
        8 => vga::print_str("Double Fault (#DF)"),
        9 => vga::print_str("Coprocessor Segment Overrun"),
        10 => vga::print_str("Invalid TSS (#TS)"),
        11 => vga::print_str("Segment Not Present (#NP)"),
        12 => vga::print_str("Stack-Segment Fault (#SS)"),
        13 => vga::print_str("General Protection Fault (#GP)"),
        14 => {
            vga::print_str("Page Fault (#PF)");
            let cr2 = unsafe { keira_arch::cpu::read_cr2() } as u64;
            vga::print_str("\nFaulting Virtual Address (CR2): 0x");
            print_hex(cr2);
        }
        16 => vga::print_str("x87 Floating-Point Exception (#MF)"),
        17 => vga::print_str("Alignment Check (#AC)"),
        18 => vga::print_str("Machine Check (#MC)"),
        19 => vga::print_str("SIMD Floating-Point Exception (#XM)"),
        20 => vga::print_str("Virtualization Exception (#VE)"),
        21 => vga::print_str("Control Protection Exception (#CP)"),
        v => {
            vga::print_str("Reserved/Unknown Vector (");
            vga::print_u64(v);
            vga::print_str(")");
        }
    }
    vga::print_str("\n");

    vga::print_str("Error Code: 0x");
    print_hex(error_code);
    vga::print_str("\n");

    vga::print_str("\nRegister Dump:\n");
    vga::print_str("  RIP: 0x");
    print_hex(rip);
    vga::print_str("   RSP: 0x");
    print_hex(rsp);
    vga::print_str("\n");
    vga::print_str("  CS:  0x");
    print_hex(cs);
    vga::print_str("   SS:  0x");
    print_hex(ss);
    vga::print_str("   RFLAGS: 0x");
    print_hex(rflags);
    vga::print_str("\n");
    vga::print_str("  RAX: 0x");
    print_hex(rax);
    vga::print_str("   RBX: 0x");
    print_hex(rbx);
    vga::print_str("\n");
    vga::print_str("  RCX: 0x");
    print_hex(rcx);
    vga::print_str("   RDX: 0x");
    print_hex(rdx);
    vga::print_str("\n");
    vga::print_str("  RSI: 0x");
    print_hex(rsi);
    vga::print_str("   RDI: 0x");
    print_hex(rdi);
    vga::print_str("\n");
    vga::print_str("  RBP: 0x");
    print_hex(rbp);
    #[cfg(target_arch = "x86_64")]
    {
        vga::print_str("   R8:  0x");
        print_hex(frame.r8);
        vga::print_str("\n");
        vga::print_str("  R9:  0x");
        print_hex(frame.r9);
        vga::print_str("   R10: 0x");
        print_hex(frame.r10);
        vga::print_str("\n");
        vga::print_str("  R11: 0x");
        print_hex(frame.r11);
        vga::print_str("   R12: 0x");
        print_hex(frame.r12);
        vga::print_str("\n");
        vga::print_str("  R13: 0x");
        print_hex(frame.r13);
        vga::print_str("   R14: 0x");
        print_hex(frame.r14);
        vga::print_str("   R15: 0x");
        print_hex(frame.r15);
    }
    vga::print_str("\n");
    vga::print_str("\nSystem halted. Please reboot/reset your computer.\n");

    serial::print_str("\n*** KERNEL PANIC ***\n");
    serial::print_str("Unhandled exception vector: ");
    print_decimal_serial(vector);
    serial::print_str("\nRIP: 0x");
    print_hex_serial(rip);
    serial::print_str("\nRSP: 0x");
    print_hex_serial(rsp);
    serial::print_str("\nError Code: 0x");
    print_hex_serial(error_code);
    serial::print_str("\n");

    unwind_from_frame(rbp, rip);

    loop {
        core::arch::asm!("cli; hlt");
    }
}

fn print_hex(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

fn print_hex_serial(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        serial::print_str(s);
    }
}

fn print_decimal_serial(val: u64) {
    if val == 0 {
        serial::print_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut temp = val;
    while temp > 0 {
        buf[i] = b'0' + (temp % 10) as u8;
        temp /= 10;
        i += 1;
    }
    for idx in 0..i {
        let char_buf = [buf[i - 1 - idx]];
        if let Ok(s) = core::str::from_utf8(&char_buf) {
            serial::print_str(s);
        }
    }
}
