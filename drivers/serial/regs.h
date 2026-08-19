/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_SERIAL_REGS_H
#define KEIRA_DRIVERS_SERIAL_REGS_H

/**
 * UART 16550A Serial Controller Registers and Bitmasks
 */

#define COM1_BASE 0x3F8

#define COM1_DATA (COM1_BASE + 0)
#define COM1_INT_ENABLE (COM1_BASE + 1)
#define COM1_FIFO_CTRL (COM1_BASE + 2)
#define COM1_LINE_CTRL (COM1_BASE + 3)
#define COM1_MODEM_CTRL (COM1_BASE + 4)
#define COM1_LINE_STATUS (COM1_BASE + 5)

#define COM1_DIVISOR_LSB (COM1_BASE + 0)
#define COM1_DIVISOR_MSB (COM1_BASE + 1)

#define LSR_TX_EMPTY 0x20

#endif /* KEIRA_DRIVERS_SERIAL_REGS_H */