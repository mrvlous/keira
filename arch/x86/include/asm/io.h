/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_ARCH_X86_ASM_IO_H
#define KEIRA_ARCH_X86_ASM_IO_H

#include <stdint.h>

/**
 * Hardware I/O Port Access Primitives
 *
 * Direct hardware port input/output inline assembly wrappers for x86 architecture.
 */

/**
 * outb - Write an 8-bit byte to an I/O port.
 * @port: 16-bit hardware I/O port address.
 * @value: 8-bit data byte to write.
 */
static inline void outb(uint16_t port, uint8_t value) {
    __asm__ volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

/**
 * inb - Read an 8-bit byte from an I/O port.
 * @port: 16-bit hardware I/O port address.
 *
 * Return: 8-bit data byte read from the port.
 */
static inline uint8_t inb(uint16_t port) {
    uint8_t result;
    __asm__ volatile("inb %1, %0" : "=a"(result) : "Nd"(port));
    return result;
}

/**
 * outl - Write a 32-bit double-word to an I/O port.
 * @port: 16-bit hardware I/O port address.
 * @value: 32-bit data word to write.
 */
static inline void outl(uint16_t port, uint32_t value) {
    __asm__ volatile("outl %0, %1" : : "a"(value), "Nd"(port));
}

/**
 * inl - Read a 32-bit double-word from an I/O port.
 * @port: 16-bit hardware I/O port address.
 *
 * Return: 32-bit data word read from the port.
 */
static inline uint32_t inl(uint16_t port) {
    uint32_t result;
    __asm__ volatile("inl %1, %0" : "=a"(result) : "Nd"(port));
    return result;
}

/**
 * io_wait - Force CPU bus wait cycle.
 *
 * Writes to unused port 0x80 to introduce microsecond delay for legacy devices.
 */
static inline void io_wait(void) {
    outb(0x80, 0);
}

#endif /* KEIRA_ARCH_X86_ASM_IO_H */