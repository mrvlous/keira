/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_ARCH_X86_ASM_IDT_H
#define KEIRA_ARCH_X86_ASM_IDT_H

#include <stdint.h>

/**
 * x86_64 Interrupt Descriptor Table (IDT) Subsystem
 *
 * Structure definitions and management primitives for hardware interrupts,
 * software interrupts, and CPU exceptions.
 */

/**
 * struct idt_entry - 64-bit Interrupt Gate Descriptor layout.
 * @offset_low: Target ISR offset bits 0..15.
 * @selector: Code segment selector in Global Descriptor Table.
 * @ist: Interrupt Stack Table index (bits 0..2).
 * @type_attr: Gate type and attributes (Present, DPL, Gate Type).
 * @offset_mid: Target ISR offset bits 16..31.
 * @offset_high: Target ISR offset bits 32..63.
 * @zero: Reserved bits, must be zero.
 */
typedef struct {
    uint16_t offset_low;
    uint16_t selector;
    uint8_t ist;
    uint8_t type_attr;
    uint16_t offset_mid;
    uint32_t offset_high;
    uint32_t zero;
} __attribute__((packed)) idt_entry_t;

/**
 * struct idt_ptr - IDTR register pointer payload format loaded via `lidt`.
 * @limit: Size of IDT array in bytes minus 1.
 * @base: Linear physical memory address of IDT array start.
 */
typedef struct {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed)) idt_ptr_t;

/**
 * idt_set_gate - Configure an individual entry in the Interrupt Descriptor Table.
 * @num: Interrupt vector index (0-255).
 * @base: Physical entry point address of Interrupt Service Routine.
 * @sel: Target code segment selector.
 * @flags: Privilege level and descriptor attribute byte.
 * @ist: Interrupt Stack Table offset.
 */
void idt_set_gate(uint8_t num, uint64_t base, uint16_t sel, uint8_t flags, uint8_t ist);

/**
 * idt_init - Initialize the hardware IDT and load register into CPU.
 */
void idt_init(void);

#endif /* KEIRA_ARCH_X86_ASM_IDT_H */
