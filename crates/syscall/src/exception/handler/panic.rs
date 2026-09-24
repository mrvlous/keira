// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Low-level kernel panic formatting and register state printing.

use keira_arch::debug::unwind::unwind_from_frame;
use keira_io::serial;
use keira_io::vga;

use crate::exception::frame::ExceptionStackFrame;

pub fn print_hex(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

pub fn print_hex_serial(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        serial::print_str(s);
    }
}

pub fn print_decimal_serial(val: u64) {
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

/// Emits kernel panic diagnostic messages to both VGA and Serial, then halts execution.
pub unsafe fn panic_exception_dump(
    vector: u64,
    error_code: u64,
    frame_ptr: *const ExceptionStackFrame,
    rip: u64,
    rsp: u64,
    rbp: u64,
    cs: u64,
    ss: u64,
    rflags: u64,
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
) -> ! {
    #[cfg(target_arch = "x86_64")]
    let frame = &*frame_ptr;
    #[cfg(not(target_arch = "x86_64"))]
    let _ = frame_ptr;

    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("\n*** KERNEL PANIC ***\n");
    vga::print_str("Unhandled CPU Exception: ");
    match vector {
        0 => vga::print_str("Divide Error (#DE)"),
        1 => vga::print_str("Debug (#DB)"),
        2 => vga::print_str("Non-Maskable Interrupt (NMI)"),
        3 => vga::print_str("Breakpoint (#BP)"),
        4 => vga::print_str("Overflow (#OF)"),
        5 => vga::print_str("Bound Range Exceeded (#BR)"),
        6 => vga::print_str("Invalid Opcode (#UD)"),
        7 => vga::print_str("Device Not Available (#NM)"),
        8 => vga::print_str("Double Fault (#DF)"),
        10 => vga::print_str("Invalid TSS (#TS)"),
        11 => vga::print_str("Segment Not Present (#NP)"),
        12 => vga::print_str("Stack-Segment Fault (#SS)"),
        13 => vga::print_str("General Protection Fault (#GP)"),
        14 => {
            vga::print_str("Page Fault (#PF)");
            #[cfg(not(test))]
            let cr2 = keira_arch::cpu::read_cr2() as u64;
            #[cfg(test)]
            let cr2 = 0u64;
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

    #[cfg(not(test))]
    loop {
        core::arch::asm!("cli; hlt");
    }
    #[cfg(test)]
    panic!("Unhandled kernel exception occurred");
}
