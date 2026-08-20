// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! x86_64 Interrupt Descriptor Table (IDT) configuration and dispatch in pure Rust.

use super::pic;

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub zero: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtPtr {
    pub limit: u16,
    pub base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 256];

static mut IDTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

extern "C" {
    fn idt_load(ptr: u64);

    fn isr32();
    fn isr33();
    fn isr44();

    fn exception0();
    fn exception1();
    fn exception2();
    fn exception3();
    fn exception4();
    fn exception5();
    fn exception6();
    fn exception7();
    fn exception8();
    fn exception9();
    fn exception10();
    fn exception11();
    fn exception12();
    fn exception13();
    fn exception14();
    fn exception15();
    fn exception16();
    fn exception17();
    fn exception18();
    fn exception19();
    fn exception20();
    fn exception21();
    fn exception22();
    fn exception23();
    fn exception24();
    fn exception25();
    fn exception26();
    fn exception27();
    fn exception28();
    fn exception29();
    fn exception30();
    fn exception31();
}

/// Set a single gate descriptor in the IDT.
pub fn set_gate(num: usize, base: u64, selector: u16, flags: u8, ist: u8) {
    if num < 256 {
        unsafe {
            IDT[num].offset_low = (base & 0xFFFF) as u16;
            IDT[num].selector = selector;
            IDT[num].ist = ist;
            IDT[num].type_attr = flags;
            IDT[num].offset_mid = ((base >> 16) & 0xFFFF) as u16;
            IDT[num].offset_high = ((base >> 32) & 0xFFFFFFFF) as u32;
            IDT[num].zero = 0;
        }
    }
}

/// Initialize the 256-entry IDT and load it into the CPU IDTR register.
pub fn init() {
    unsafe {
        IDTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDTR.base = core::ptr::addr_of!(IDT) as u64;

        for i in 0..256 {
            set_gate(i, 0, 0, 0, 0);
        }

        let exceptions: [unsafe extern "C" fn(); 32] = [
            exception0,
            exception1,
            exception2,
            exception3,
            exception4,
            exception5,
            exception6,
            exception7,
            exception8,
            exception9,
            exception10,
            exception11,
            exception12,
            exception13,
            exception14,
            exception15,
            exception16,
            exception17,
            exception18,
            exception19,
            exception20,
            exception21,
            exception22,
            exception23,
            exception24,
            exception25,
            exception26,
            exception27,
            exception28,
            exception29,
            exception30,
            exception31,
        ];

        for (i, &handler) in exceptions.iter().enumerate() {
            set_gate(i, handler as usize as u64, 0x08, 0x8E, 0);
        }

        // Hardware IRQ gates (Code selector 0x08, 64-bit Interrupt Gate 0x8E)
        set_gate(32, isr32 as *const () as usize as u64, 0x08, 0x8E, 0);
        set_gate(33, isr33 as *const () as usize as u64, 0x08, 0x8E, 0);
        set_gate(44, isr44 as *const () as usize as u64, 0x08, 0x8E, 0);

        idt_load(core::ptr::addr_of!(IDTR) as u64);
    }
}

/// Generic ISR dispatcher called from assembly stubs if required.
#[no_mangle]
pub extern "C" fn isr_handler(vector: u64) {
    if vector == 32 {
        crate::timers::pit::pit_handler();
    } else if vector >= 40 {
        pic::send_eoi(8);
    } else {
        pic::send_eoi(0);
    }
}
