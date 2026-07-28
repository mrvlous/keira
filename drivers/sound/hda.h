/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_SOUND_HDA_H
#define KEIRA_DRIVERS_SOUND_HDA_H

#include <stdint.h>

/**
 * Intel High Definition Audio (HDA) Controller Interface
 */

/**
 * hda_init - Map MMIO registers and initialize Intel HDA controller.
 * @bar_phys: Physical address of HDA MMIO Base Address Register (BAR).
 */
void hda_init(uint64_t bar_phys);

/**
 * hda_start_tone - Start DMA audio playback stream at specified frequency.
 * @bdl_phys: Physical address of Buffer Descriptor List payload.
 * @buf1_phys: Physical address of first double-buffer page.
 * @buf2_phys: Physical address of second double-buffer page.
 * @freq: Target audio tone frequency in Hertz.
 */
void hda_start_tone(uint64_t bdl_phys, uint64_t buf1_phys, uint64_t buf2_phys, uint32_t freq);

/**
 * hda_stop - Stop active HDA DMA output stream.
 */
void hda_stop(void);

#endif /* KEIRA_DRIVERS_SOUND_HDA_H */
