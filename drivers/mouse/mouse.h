/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_MOUSE_MOUSE_H
#define KEIRA_DRIVERS_MOUSE_MOUSE_H

#include <stdint.h>

/**
 * PS/2 Auxiliary Mouse Driver Interface
 *
 * Initializes the 8042 PS/2 auxiliary controller and processes 3-byte packets
 * for screen cursor tracking.
 */

/**
 * mouse_init - Initialize PS/2 auxiliary mouse controller.
 */
void mouse_init(void);

/**
 * mouse_handler - IRQ12 mouse interrupt service routine handler.
 */
void mouse_handler(void);

/**
 * mouse_set_resolution - Configure mouse motion coordinate limits.
 * @width: Target screen width in pixels or character cells.
 * @height: Target screen height in pixels or character cells.
 */
void mouse_set_resolution(int32_t width, int32_t height);

#endif /* KEIRA_DRIVERS_MOUSE_MOUSE_H */
