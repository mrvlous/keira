/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_KEYBOARD_KEYBOARD_H
#define KEIRA_DRIVERS_KEYBOARD_KEYBOARD_H

#include <stdint.h>

/**
 * PS/2 Keyboard Peripheral Driver Interface
 *
 * Handles IRQ1 keyboard hardware interrupts, converts scan codes to ASCII,
 * and routes key events to the shell subsystem.
 */

/**
 * keyboard_init - Initialize PS/2 keyboard hardware driver.
 */
void keyboard_init(void);

/**
 * keyboard_handler - IRQ1 keyboard interrupt service routine handler.
 */
void keyboard_handler(void);

#endif /* KEIRA_DRIVERS_KEYBOARD_KEYBOARD_H */