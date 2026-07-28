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
 * 8253/8254 Programmable Interval Timer (PIT) Driver
 *
 * Configures Channel 0 timer square-wave oscillator frequency and tracks
 * cumulative system tick counters.
 */

#include <asm/io.h>
#include <asm/pic.h>
#include <asm/pit.h>
#include <stddef.h>
#include <stdint.h>

#define PIT_CMD_PORT 0x43
#define PIT_CH0_PORT 0x40
#define PIT_BASE_FREQ 1193180

static volatile uint64_t timer_ticks_ms = 0;

/**
 * pit_init - Program PIT oscillator frequency divisor.
 * @frequency: Target interrupt frequency in Hz.
 */
void pit_init(uint32_t frequency) {
    if (frequency == 0) {
        return;
    }

    uint32_t divisor = PIT_BASE_FREQ / frequency;
    if (divisor > 65535) {
        divisor = 65535;
    }
    if (divisor == 0) {
        divisor = 1;
    }

    outb(PIT_CMD_PORT, 0x36);

    uint8_t l = (uint8_t)(divisor & 0xFF);
    uint8_t h = (uint8_t)((divisor >> 8) & 0xFF);

    outb(PIT_CH0_PORT, l);
    outb(PIT_CH0_PORT, h);

    pic_clear_mask(0);
}

/**
 * pit_handler - System tick interrupt service handler routine.
 */
void pit_handler(void) {
    timer_ticks_ms++;
    pic_eoi(0);
}

/**
 * get_uptime_ms - Read uptime duration in milliseconds.
 *
 * Return: Milliseconds elapsed since boot time.
 */
uint64_t get_uptime_ms(void) {
    return timer_ticks_ms;
}
