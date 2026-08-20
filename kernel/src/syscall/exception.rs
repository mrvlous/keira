// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides the Rust handler dispatcher for CPU exceptions.
//! When a CPU exception occurs, we dump the registers and halt the CPU.

use crate::io::vga;

#[repr(C, packed)]
pub struct ExceptionStackFrame {
    // Saved by pushaq
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
    // Pushed by stub
    pub vector: u64,
    pub error_code: u64,
    // Pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

/// CPU exception dispatcher invoked by low-level assembly ISR stubs.
///
/// # Safety
/// `frame_ptr` must be a valid, aligned pointer to an `ExceptionStackFrame` constructed by the CPU/ISR.
#[no_mangle]
pub unsafe extern "C" fn exception_dispatcher(frame_ptr: *const ExceptionStackFrame) {
    let frame = &*frame_ptr;

    // Check if the exception occurred in User Mode (Ring 3)
    if (frame.cs & 3) == 3 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("\n*** USER PROCESS CRASHED ***\n");
        vga::print_str("Exception Vector: ");
        vga::print_u64(frame.vector);
        if frame.vector == 14 {
            vga::print_str(" (Page Fault)");
        } else if frame.vector == 13 {
            vga::print_str(" (General Protection Fault)");
        }
        vga::print_str("\nRIP: 0x");
        print_hex(frame.rip);
        if frame.vector == 14 {
            let cr2: u64;
            core::arch::asm!("mov {}, cr2", out(reg) cr2);
            vga::print_str(" | Faulting Address: 0x");
            print_hex(cr2);
        }
        vga::print_str("\nTerminating crashed user process...\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        if crate::task::scheduler::CURRENT_TASK_IDX != 0 {
            crate::task::scheduler::exit_current();
        } else {
            extern "C" {
                fn abort_user_mode() -> !;
            }
            abort_user_mode();
        }
    }

    // Set VGA color to red on black for Kernel Panic
    vga::set_color(vga::Color::LightRed, vga::Color::Black);

    vga::print_str("\n*** KERNEL PANIC ***\n");

    // Print Exception details
    vga::print_str("UNHANDLED CPU EXCEPTION: ");
    match frame.vector {
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
            // Read CR2 register (contains the faulting address)
            let cr2: u64;
            unsafe {
                core::arch::asm!("mov {}, cr2", out(reg) cr2);
            }
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
    print_hex(frame.error_code);
    vga::print_str("\n");

    // Print register dumps
    vga::print_str("\nRegister Dump:\n");
    vga::print_str("  RIP: 0x");
    print_hex(frame.rip);
    vga::print_str("   RSP: 0x");
    print_hex(frame.rsp);
    vga::print_str("\n");
    vga::print_str("  CS:  0x");
    print_hex(frame.cs);
    vga::print_str("   SS:  0x");
    print_hex(frame.ss);
    vga::print_str("   RFLAGS: 0x");
    print_hex(frame.rflags);
    vga::print_str("\n");
    vga::print_str("  RAX: 0x");
    print_hex(frame.rax);
    vga::print_str("   RBX: 0x");
    print_hex(frame.rbx);
    vga::print_str("\n");
    vga::print_str("  RCX: 0x");
    print_hex(frame.rcx);
    vga::print_str("   RDX: 0x");
    print_hex(frame.rdx);
    vga::print_str("\n");
    vga::print_str("  RSI: 0x");
    print_hex(frame.rsi);
    vga::print_str("   RDI: 0x");
    print_hex(frame.rdi);
    vga::print_str("\n");
    vga::print_str("  RBP: 0x");
    print_hex(frame.rbp);
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
    vga::print_str("\n");
    vga::print_str("\nSystem halted. Please reboot/reset your computer.\n");

    // Output to serial port (COM1) for host debugging
    crate::io::serial::print_str("\n*** KERNEL PANIC ***\n");
    crate::io::serial::print_str("Unhandled exception vector: ");
    print_decimal_serial(frame.vector);
    crate::io::serial::print_str("\nRIP: 0x");
    print_hex_serial(frame.rip);
    crate::io::serial::print_str("\nRSP: 0x");
    print_hex_serial(frame.rsp);
    crate::io::serial::print_str("\nError Code: 0x");
    print_hex_serial(frame.error_code);
    crate::io::serial::print_str("\n");

    // Walk stack frame pointers and output callstack backtrace
    crate::arch::unwind::unwind_from_frame(frame.rbp, frame.rip);

    // Halt the CPU
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
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
        crate::io::serial::print_str(s);
    }
}

fn print_decimal_serial(val: u64) {
    if val == 0 {
        crate::io::serial::print_str("0");
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
            crate::io::serial::print_str(s);
        }
    }
}
