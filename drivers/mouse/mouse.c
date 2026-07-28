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
 * PS/2 Mouse Driver Implementation
 */

#include "mouse.h"

#include "regs.h"

#include <asm/io.h>
#include <asm/pic.h>
#include <vga/vga.h>

static uint8_t mouse_cycle = 0;
static int8_t mouse_byte[3];

static int32_t mouse_x = 40;
static int32_t mouse_y = 12;
static int32_t mouse_fx = 40 * 256;
static int32_t mouse_fy = 12 * 256;

static int32_t mouse_max_x = 80;
static int32_t mouse_max_y = 25;
static int32_t mouse_sensitivity_x = 24;
static int32_t mouse_sensitivity_y = 12;

/**
 * mouse_set_resolution - Configure mouse motion coordinate limits.
 * @width: Target screen width in pixels or character cells.
 * @height: Target screen height in pixels or character cells.
 */
void mouse_set_resolution(int32_t width, int32_t height) {
    mouse_max_x = width;
    mouse_max_y = height;
    mouse_x = width / 2;
    mouse_y = height / 2;
    mouse_fx = mouse_x * 256;
    mouse_fy = mouse_y * 256;

    if (width > 80) {
        mouse_sensitivity_x = 512;
        mouse_sensitivity_y = 512;
    } else {
        mouse_sensitivity_x = 24;
        mouse_sensitivity_y = 12;
    }
}

/**
 * mouse_wait - Block wait loop for PS/2 input buffer empty or output buffer full.
 * @a_type: Wait condition selector (0: output buffer full, 1: input buffer empty).
 */
static inline void mouse_wait(uint8_t a_type) {
    uint32_t timeout = 100000;
    if (a_type == 0) {
        while (timeout--) {
            if ((inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) == PS2_STATUS_OUTPUT_FULL) {
                return;
            }
        }
    } else {
        while (timeout--) {
            if ((inb(PS2_STATUS_PORT) & PS2_STATUS_INPUT_FULL) == 0) {
                return;
            }
        }
    }
}

/**
 * mouse_write - Write command byte to the PS/2 auxiliary device port.
 * @a_write: Command data byte.
 */
static inline void mouse_write(uint8_t a_write) {
    mouse_wait(1);
    outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_MOUSE);
    mouse_wait(1);
    outb(PS2_DATA_PORT, a_write);
}

/**
 * mouse_read - Read response byte from the PS/2 auxiliary data port.
 *
 * Return: Status/data byte read.
 */
static inline uint8_t mouse_read(void) {
    mouse_wait(0);
    return inb(PS2_DATA_PORT);
}

/**
 * mouse_init - Initialize PS/2 auxiliary mouse controller.
 */
void mouse_init(void) {
    uint8_t status;

    mouse_wait(1);
    outb(PS2_COMMAND_PORT, PS2_CMD_ENABLE_MOUSE);

    mouse_wait(1);
    outb(PS2_COMMAND_PORT, PS2_CMD_READ_CONFIG);
    mouse_wait(0);
    status = inb(PS2_DATA_PORT);

    status |= 2;
    status &= ~0x20;

    mouse_wait(1);
    outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_CONFIG);
    mouse_wait(1);
    outb(PS2_DATA_PORT, status);

    mouse_write(MOUSE_CMD_SET_DEFAULTS);
    mouse_read();

    mouse_write(MOUSE_CMD_ENABLE_STREAM);
    mouse_read();

    pic_clear_mask(MOUSE_IRQ);

    vga_draw_mouse(mouse_x, mouse_y);
}

/**
 * mouse_handler - IRQ12 mouse interrupt service routine handler.
 */
void mouse_handler(void) {
    uint8_t status = inb(PS2_STATUS_PORT);

    if ((status & PS2_STATUS_MOUSE_DATA) == PS2_STATUS_MOUSE_DATA &&
        (status & PS2_STATUS_OUTPUT_FULL) == PS2_STATUS_OUTPUT_FULL) {
        mouse_byte[mouse_cycle++] = inb(PS2_DATA_PORT);

        if (mouse_cycle == 1 && (mouse_byte[0] & MOUSE_FLAGS_SYNC) == 0) {
            mouse_cycle = 0;
            pic_eoi(MOUSE_IRQ);
            return;
        }

        if (mouse_cycle == 3) {
            mouse_cycle = 0;

            if ((mouse_byte[0] & (MOUSE_FLAGS_X_OVERFLOW | MOUSE_FLAGS_Y_OVERFLOW)) == 0) {
                mouse_fx += (int32_t)mouse_byte[1] * mouse_sensitivity_x;
                mouse_fy -= (int32_t)mouse_byte[2] * mouse_sensitivity_y;

                if (mouse_fx < 0) {
                    mouse_fx = 0;
                }
                if (mouse_fx >= mouse_max_x * 256) {
                    mouse_fx = (mouse_max_x - 1) * 256;
                }
                if (mouse_fy < 0) {
                    mouse_fy = 0;
                }
                if (mouse_fy >= mouse_max_y * 256) {
                    mouse_fy = (mouse_max_y - 1) * 256;
                }

                int32_t new_x = mouse_fx / 256;
                int32_t new_y = mouse_fy / 256;

                if (new_x != mouse_x || new_y != mouse_y) {
                    vga_clear_mouse(mouse_x, mouse_y);

                    mouse_x = new_x;
                    mouse_y = new_y;

                    vga_draw_mouse(mouse_x, mouse_y);
                }
            }
        }
    } else {
        if (status & PS2_STATUS_OUTPUT_FULL) {
            inb(PS2_DATA_PORT);
        }
    }

    pic_eoi(MOUSE_IRQ);
}
