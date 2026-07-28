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
 * Intel High Definition Audio (HDA) Driver Implementation
 */

#include "hda.h"

#define HDA_REG_GCTL 0x08
#define HDA_REG_STATESTS 0x0E
#define HDA_REG_IC 0x60
#define HDA_REG_IR 0x64
#define HDA_REG_ICS 0x68

#define SD_BASE 0x100
#define SD_REG_CTL (SD_BASE + 0x00)
#define SD_REG_STS (SD_BASE + 0x03)
#define SD_REG_CBL (SD_BASE + 0x08)
#define SD_REG_LVI (SD_BASE + 0x0C)
#define SD_REG_FMTS (SD_BASE + 0x12)
#define SD_REG_BDPL (SD_BASE + 0x18)
#define SD_REG_BDPU (SD_BASE + 0x1C)

/**
 * struct hda_bdl_entry - Intel HDA DMA Buffer Descriptor List Entry.
 * @phys_low: Lower 32 bits of buffer page physical memory address.
 * @phys_high: Upper 32 bits of buffer page physical memory address.
 * @length: Length of buffer page in bytes.
 * @flags: Buffer control flags (e.g. Interrupt on Complete).
 */
struct hda_bdl_entry {
    uint32_t phys_low;
    uint32_t phys_high;
    uint32_t length;
    uint32_t flags;
} __attribute__((packed));

static uint64_t hda_base_virt = 0;

/**
 * delay - Wait loop for microsecond-scale delay.
 * @count: Loop iteration count.
 */
static void delay(int count) {
    volatile int i = count;
    while (i > 0) {
        i--;
    }
}

/**
 * hda_send_verb - Send a control verb to codec and wait for response.
 * @codec: Codec address index (typically 0).
 * @node: Target widget/node index.
 * @verb_param: Verb command parameter payload.
 *
 * Return: Response payload from the codec.
 */
static uint32_t hda_send_verb(uint8_t codec, uint8_t node, uint32_t verb_param) {
    if (!hda_base_virt) {
        return 0;
    }

    volatile uint32_t *ic = (volatile uint32_t *)(hda_base_virt + HDA_REG_IC);
    volatile uint32_t *ir = (volatile uint32_t *)(hda_base_virt + HDA_REG_IR);
    volatile uint16_t *ics = (volatile uint16_t *)(hda_base_virt + HDA_REG_ICS);

    int timeout = 100000;
    while ((*ics & 1) && timeout > 0) {
        timeout--;
    }
    if (timeout == 0) {
        return 0;
    }

    *ics = *ics | 2;

    uint32_t command = ((uint32_t)codec << 28) | ((uint32_t)node << 20) | (verb_param & 0xFFFFF);
    *ic = command;

    *ics = (*ics & ~2) | 1;

    timeout = 100000;
    while ((*ics & 1) && timeout > 0) {
        timeout--;
    }
    if (timeout == 0) {
        return 0;
    }

    return *ir;
}

/**
 * hda_init - Map MMIO registers and initialize Intel HDA controller.
 * @bar_phys: Physical address of HDA MMIO Base Address Register (BAR).
 */
void hda_init(uint64_t bar_phys) {
    hda_base_virt = bar_phys;

    volatile uint32_t *gctl = (volatile uint32_t *)(hda_base_virt + HDA_REG_GCTL);

    *gctl &= ~1;
    int timeout = 100000;
    while ((*gctl & 1) && timeout > 0) {
        timeout--;
    }
    delay(10000);

    *gctl |= 1;
    timeout = 100000;
    while (!(*gctl & 1) && timeout > 0) {
        timeout--;
    }
    delay(10000);

    hda_send_verb(0, 0x03, (0x707 << 8) | 0x40);
    hda_send_verb(0, 0x03, (0x701 << 8) | 0x00);
    hda_send_verb(0, 0x02, (0x3 << 16) | 0xB07F);
    hda_send_verb(0, 0x03, (0x3 << 16) | 0xB07F);
}

/**
 * hda_fill_square - Synthesize a stereo square-wave audio frame.
 * @buf: Target PCM buffer.
 * @count: Sample frame count.
 * @freq: Target wave frequency in Hertz.
 */
static void hda_fill_square(int16_t *buf, int count, int freq) {
    int sample_rate = 48000;
    int period = sample_rate / freq;
    int half_period = period / 2;

    for (int i = 0; i < count; i++) {
        int phase = i % period;
        int16_t val = (phase < half_period) ? 8000 : -8000;
        buf[2 * i] = val;
        buf[2 * i + 1] = val;
    }
}

/**
 * hda_start_tone - Start DMA audio playback stream at specified frequency.
 * @bdl_phys: Physical address of Buffer Descriptor List payload.
 * @buf1_phys: Physical address of first double-buffer page.
 * @buf2_phys: Physical address of second double-buffer page.
 * @freq: Target audio tone frequency in Hertz.
 */
void hda_start_tone(uint64_t bdl_phys, uint64_t buf1_phys, uint64_t buf2_phys, uint32_t freq) {
    if (!hda_base_virt) {
        return;
    }

    hda_fill_square((int16_t *)buf1_phys, 1024, freq);
    hda_fill_square((int16_t *)buf2_phys, 1024, freq);

    struct hda_bdl_entry *bdl = (struct hda_bdl_entry *)bdl_phys;

    bdl[0].phys_low = (uint32_t)buf1_phys;
    bdl[0].phys_high = (uint32_t)(buf1_phys >> 32);
    bdl[0].length = 4096;
    bdl[0].flags = 1;

    bdl[1].phys_low = (uint32_t)buf2_phys;
    bdl[1].phys_high = (uint32_t)(buf2_phys >> 32);
    bdl[1].length = 4096;
    bdl[1].flags = 1;

    hda_send_verb(0, 0x02, (0x2 << 16) | 0x0011);
    hda_send_verb(0, 0x02, (0x706 << 8) | 0x10);

    volatile uint32_t *sd_ctl = (volatile uint32_t *)(hda_base_virt + SD_REG_CTL);
    volatile uint8_t *sd_sts = (volatile uint8_t *)(hda_base_virt + SD_REG_STS);
    volatile uint32_t *sd_cbl = (volatile uint32_t *)(hda_base_virt + SD_REG_CBL);
    volatile uint16_t *sd_lvi = (volatile uint16_t *)(hda_base_virt + SD_REG_LVI);
    volatile uint16_t *sd_fmts = (volatile uint16_t *)(hda_base_virt + SD_REG_FMTS);
    volatile uint32_t *sd_bdpl = (volatile uint32_t *)(hda_base_virt + SD_REG_BDPL);
    volatile uint32_t *sd_bdpu = (volatile uint32_t *)(hda_base_virt + SD_REG_BDPU);

    *sd_ctl &= ~2;
    int timeout = 100000;
    while ((*sd_ctl & 2) && timeout > 0) {
        timeout--;
    }

    *sd_bdpl = (uint32_t)bdl_phys;
    *sd_bdpu = (uint32_t)(bdl_phys >> 32);
    *sd_cbl = 8192;
    *sd_lvi = 1;
    *sd_fmts = 0x0011;
    *sd_sts = 0xFF;

    uint32_t ctrl = *sd_ctl;
    ctrl &= ~(0xF << 20);
    ctrl |= (1 << 20);
    *sd_ctl = ctrl;

    *sd_ctl |= 2;
}

/**
 * hda_stop - Stop active HDA DMA output stream.
 */
void hda_stop(void) {
    if (!hda_base_virt) {
        return;
    }

    volatile uint32_t *sd_ctl = (volatile uint32_t *)(hda_base_virt + SD_REG_CTL);
    *sd_ctl &= ~2;
}
