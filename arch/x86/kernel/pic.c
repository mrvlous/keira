/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Dual 8259 Programmable Interrupt Controller Implementation
 *
 * Driver routines for cascade initialization, vector remapping, masking,
 * and End-of-Interrupt (EOI) signaling.
 */

#include <asm/io.h>
#include <asm/pic.h>
#include <stddef.h>
#include <stdint.h>

#define PIC1_COMMAND 0x20
#define PIC1_DATA 0x21
#define PIC2_COMMAND 0xA0
#define PIC2_DATA 0xA1

#define ICW1_ICW4 0x01
#define ICW1_INIT 0x10
#define ICW4_8086 0x01

#define PIC_EOI 0x20

/**
 * pic_init - Execute cascade initialization sequence on 8259 PIC pair.
 * @offset1: IRQ base offset for Master PIC (0..7 mapped to offset1..offset1+7).
 * @offset2: IRQ base offset for Slave PIC (8..15 mapped to offset2..offset2+7).
 */
void pic_init(int offset1, int offset2) {
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    outb(PIC1_DATA, offset1);
    io_wait();
    outb(PIC2_DATA, offset2);
    io_wait();

    outb(PIC1_DATA, 4);
    io_wait();
    outb(PIC2_DATA, 2);
    io_wait();

    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    outb(PIC1_DATA, 0xFF);
    outb(PIC2_DATA, 0xFF);

    pic_clear_mask(2);
}

/**
 * pic_eoi - Signal End-of-Interrupt command byte to hardware PIC controller.
 * @irq: Hardware IRQ line number (0-15).
 */
void pic_eoi(unsigned char irq) {
    if (irq >= 8) {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

/**
 * pic_set_mask - Set mask bit to disable interrupt signal on target IRQ line.
 * @irqline: Hardware IRQ line number (0-15).
 */
void pic_set_mask(unsigned char irqline) {
    uint16_t port;
    uint8_t value;

    if (irqline < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irqline -= 8;
    }
    value = inb(port) | (1 << irqline);
    outb(port, value);
}

/**
 * pic_clear_mask - Clear mask bit to enable interrupt signal on target IRQ line.
 * @irqline: Hardware IRQ line number (0-15).
 */
void pic_clear_mask(unsigned char irqline) {
    uint16_t port;
    uint8_t value;

    if (irqline < 8) {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irqline -= 8;
    }
    value = inb(port) & ~(1 << irqline);
    outb(port, value);
}