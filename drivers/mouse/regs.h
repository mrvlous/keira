/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_MOUSE_REGS_H
#define KEIRA_DRIVERS_MOUSE_REGS_H

/**
 * PS/2 8042 Controller Ports and Status Bitmasks
 */

#define PS2_DATA_PORT 0x60
#define PS2_STATUS_PORT 0x64
#define PS2_COMMAND_PORT 0x64

#define PS2_STATUS_OUTPUT_FULL 0x01
#define PS2_STATUS_INPUT_FULL 0x02
#define PS2_STATUS_MOUSE_DATA 0x20

#define PS2_CMD_READ_CONFIG 0x20
#define PS2_CMD_WRITE_CONFIG 0x60
#define PS2_CMD_ENABLE_MOUSE 0xA8
#define PS2_CMD_WRITE_MOUSE 0xD4

#define MOUSE_CMD_SET_DEFAULTS 0xF6
#define MOUSE_CMD_ENABLE_STREAM 0xF4

#define MOUSE_IRQ 12

#define MOUSE_FLAGS_LEFT_BUTTON 0x01
#define MOUSE_FLAGS_RIGHT_BUTTON 0x02
#define MOUSE_FLAGS_MIDDLE_BUTTON 0x04
#define MOUSE_FLAGS_SYNC 0x08
#define MOUSE_FLAGS_X_SIGN 0x10
#define MOUSE_FLAGS_Y_SIGN 0x20
#define MOUSE_FLAGS_X_OVERFLOW 0x40
#define MOUSE_FLAGS_Y_OVERFLOW 0x80

#endif /* KEIRA_DRIVERS_MOUSE_REGS_H */
