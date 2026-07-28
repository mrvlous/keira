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
 * PC Speaker Sound Driver Implementation
 */

#include "sound.h"

#include "regs.h"

#include <asm/io.h>

/**
 * sound_play - Program PIT Channel 2 to generate square-wave audio frequency.
 * @freq: Target frequency in Hertz.
 */
void sound_play(uint32_t freq) {
    if (freq == 0) {
        return;
    }

    uint32_t div = PIT_BASE_FREQ / freq;

    outb(PIT_CMD, PIT_CH2_MODE3);
    outb(PIT_CH2_DATA, (uint8_t)(div & 0xFF));
    outb(PIT_CH2_DATA, (uint8_t)((div >> 8) & 0xFF));

    uint8_t ctrl = inb(SYS_CTRL_B);
    if ((ctrl & SPKR_ENABLE) != SPKR_ENABLE) {
        outb(SYS_CTRL_B, ctrl | SPKR_ENABLE);
    }
}

/**
 * sound_stop - Disable PC Speaker audio output.
 */
void sound_stop(void) {
    uint8_t ctrl = inb(SYS_CTRL_B);
    outb(SYS_CTRL_B, ctrl & SPKR_DISABLE);
}
