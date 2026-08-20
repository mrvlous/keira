// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dual 8259 Programmable Interrupt Controller (PIC) driver in pure Rust.
//!
//! Provides cascade initialization, vector remapping, masking, and End-of-Interrupt (EOI) signaling.

use crate::cpu::{inb, io_wait, outb};

pub const PIC1_COMMAND: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_COMMAND: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;

/// Execute cascade initialization sequence on the 8259 PIC pair.
///
/// * `offset1`: IRQ base offset for Master PIC (typically 32).
/// * `offset2`: IRQ base offset for Slave PIC (typically 40).
pub fn init(offset1: u8, offset2: u8) {
    unsafe {
        // ICW1: Start initialization in cascade mode
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // ICW2: Vector offset
        outb(PIC1_DATA, offset1);
        io_wait();
        outb(PIC2_DATA, offset2);
        io_wait();

        // ICW3: Cascade wiring (Master: slave at IRQ2 / bit 2 = 4; Slave: cascade identity 2)
        outb(PIC1_DATA, 4);
        io_wait();
        outb(PIC2_DATA, 2);
        io_wait();

        // ICW4: 8086/88 mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Mask all interrupts initially
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);

        // Unmask IRQ2 (slave PIC line on master PIC)
        clear_mask(2);
    }
}

/// Signal End-of-Interrupt (EOI) command byte to hardware PIC controller.
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

/// Set mask bit to disable interrupt signal on target IRQ line.
pub fn set_mask(mut irq: u8) {
    unsafe {
        let port = if irq < 8 {
            PIC1_DATA
        } else {
            irq -= 8;
            PIC2_DATA
        };
        let value = inb(port) | (1 << irq);
        outb(port, value);
    }
}

/// Clear mask bit to enable interrupt signal on target IRQ line.
pub fn clear_mask(mut irq: u8) {
    unsafe {
        let port = if irq < 8 {
            PIC1_DATA
        } else {
            irq -= 8;
            PIC2_DATA
        };
        let value = inb(port) & !(1 << irq);
        outb(port, value);
    }
}

// C-compatible export for assembly/legacy compatibility
#[no_mangle]
pub extern "C" fn pic_eoi(irq: u8) {
    send_eoi(irq);
}

#[no_mangle]
pub extern "C" fn pic_clear_mask(irq: u8) {
    clear_mask(irq);
}

#[no_mangle]
pub extern "C" fn pic_set_mask(irq: u8) {
    set_mask(irq);
}
