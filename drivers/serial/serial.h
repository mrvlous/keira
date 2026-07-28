/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_SERIAL_SERIAL_H
#define KEIRA_DRIVERS_SERIAL_SERIAL_H

#include <stdint.h>

/**
 * COM1 UART 16550A Serial Driver Interface
 */

/**
 * serial_init - Initialize COM1 UART to 38400 baud, 8N1 configuration.
 */
void serial_init(void);

/**
 * serial_putchar - Write a single character byte to COM1 serial port.
 * @c: ASCII character to transmit.
 */
void serial_putchar(char c);

/**
 * serial_print - Write a null-terminated string payload to COM1 serial port.
 * @str: Pointer to null-terminated ASCII string buffer.
 */
void serial_print(const char *str);

#endif /* KEIRA_DRIVERS_SERIAL_SERIAL_H */
