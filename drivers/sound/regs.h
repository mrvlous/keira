/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_SOUND_REGS_H
#define KEIRA_DRIVERS_SOUND_REGS_H

/**
 * PC Speaker Hardware Registers and Bitmasks
 */

#define PIT_CMD 0x43
#define PIT_CH2_DATA 0x42
#define SYS_CTRL_B 0x61

#define PIT_CH2_MODE3 0xB6
#define PIT_BASE_FREQ 1193182

#define SPKR_GATE_EN 0x01
#define SPKR_DATA_EN 0x02
#define SPKR_ENABLE (SPKR_GATE_EN | SPKR_DATA_EN)
#define SPKR_DISABLE 0xFC

#endif /* KEIRA_DRIVERS_SOUND_REGS_H */
