/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_ARCH_X86_ASM_PIC_H
#define KEIRA_ARCH_X86_ASM_PIC_H

#include <stdint.h>

/**
 * Dual Programmable Interrupt Controller (8259 PIC) Driver
 *
 * Manages legacy 8259 master and slave PIC irq remapping, masking,
 * and End of Interrupt (EOI) signaling.
 */

/**
 * pic_init - Initialize master and slave 8259 PICs and remap IRQs.
 * @offset1: Vector base offset for Master PIC (typically 32).
 * @offset2: Vector base offset for Slave PIC (typically 40).
 */
void pic_init(int offset1, int offset2);

/**
 * pic_eoi - Send End of Interrupt (EOI) command to PIC hardware.
 * @irq: IRQ line number (0-15) being acknowledged.
 */
void pic_eoi(unsigned char irq);

/**
 * pic_set_mask - Mask (disable) a specific IRQ line on the PIC.
 * @irqline: IRQ line number (0-15) to mask.
 */
void pic_set_mask(unsigned char irqline);

/**
 * pic_clear_mask - Unmask (enable) a specific IRQ line on the PIC.
 * @irqline: IRQ line number (0-15) to unmask.
 */
void pic_clear_mask(unsigned char irqline);

#endif /* KEIRA_ARCH_X86_ASM_PIC_H */