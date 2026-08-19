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
 * COM1 UART 16550A Serial Driver Implementation
 */

#include "serial.h"

#include "regs.h"

#include <asm/io.h>

/**
 * serial_is_tx_ready - Check if COM1 serial transmit buffer is empty.
 *
 * Return: Non-zero if ready to transmit, 0 otherwise.
 */
static int serial_is_tx_ready(void) {
    return inb(COM1_LINE_STATUS) & LSR_TX_EMPTY;
}

/**
 * serial_init - Initialize COM1 UART 16550A serial port device.
 */
void serial_init(void) {
    outb(COM1_INT_ENABLE, 0x00);
    outb(COM1_LINE_CTRL, 0x80);
    outb(COM1_DIVISOR_LSB, 0x03);
    outb(COM1_DIVISOR_MSB, 0x00);
    outb(COM1_LINE_CTRL, 0x03);
    outb(COM1_FIFO_CTRL, 0xC7);
    outb(COM1_MODEM_CTRL, 0x0B);
}

/**
 * serial_putchar - Output a single character byte to COM1 serial port.
 * @c: Character byte to transmit.
 */
void serial_putchar(char c) {
    if (c == '\n') {
        while (!serial_is_tx_ready()) {
        }
        outb(COM1_DATA, '\r');
    }

    while (!serial_is_tx_ready()) {
    }
    outb(COM1_DATA, (uint8_t)c);
}

/**
 * serial_print - Output a null-terminated string to COM1 serial port.
 * @str: Pointer to null-terminated string.
 */
void serial_print(const char *str) {
    while (*str) {
        serial_putchar(*str);
        str++;
    }
}