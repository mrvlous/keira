/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_ARCH_X86_ASM_PIT_H
#define KEIRA_ARCH_X86_ASM_PIT_H

#include <stdint.h>

/**
 * 8253/8254 Programmable Interval Timer (PIT) Subsystem Driver
 */

/**
 * pit_init - Configure PIT Channel 0 operating frequency.
 * @frequency: Desired timer interrupt tick rate in Hertz (e.g. 1000 for 1 ms).
 */
void pit_init(uint32_t frequency);

/**
 * pit_handler - Primary IRQ0 interrupt handler service routine.
 */
void pit_handler(void);

/**
 * get_uptime_ms - Retrieve cumulative system uptime.
 *
 * Return: System uptime counter value in milliseconds.
 */
uint64_t get_uptime_ms(void);

#endif /* KEIRA_ARCH_X86_ASM_PIT_H */