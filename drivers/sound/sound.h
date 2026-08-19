/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_SOUND_SOUND_H
#define KEIRA_DRIVERS_SOUND_SOUND_H

#include <stdint.h>

/**
 * PC Speaker Sound Subsystem Interface
 */

/**
 * sound_play - Output square wave tone at target frequency via PC Speaker.
 * @freq: Output tone frequency in Hertz.
 */
void sound_play(uint32_t freq);

/**
 * sound_stop - Silence PC Speaker output.
 */
void sound_stop(void);

#endif /* KEIRA_DRIVERS_SOUND_SOUND_H */