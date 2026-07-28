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
 * Early Architecture Hardware Initialization
 *
 * Called from assembly entry trampoline (_start64) to initialize critical
 * core architecture state, video text framebuffers, timers, serial output,
 * and memory allocators prior to invoking kernel_main().
 */

#include <asm/idt.h>
#include <asm/pic.h>
#include <asm/pit.h>
#include <keira/heap.h>
#include <keyboard/keyboard.h>
#include <mouse/mouse.h>
#include <rtc/rtc.h>
#include <serial/serial.h>
#include <stddef.h>
#include <stdint.h>
#include <vga/vga.h>

extern uint8_t __heap_start;
extern uint8_t __heap_end;

/**
 * print_boot_log - Format and render system boot initialization status lines.
 * @msg: Text description of initialization milestone.
 * @status: Completion code (0: OK, 1: WARN, >1: FAIL).
 */
static void print_boot_log(const char *msg, int status) {
    vga_set_color(VGA_COLOR_LIGHT_BLUE, VGA_COLOR_BLACK);
    vga_print(":: ");

    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
    vga_print(msg);

    int len = 0;
    while (msg[len]) {
        len++;
    }
    int padding = 72 - 3 - len;
    if (padding < 1) {
        padding = 1;
    }
    for (int i = 0; i < padding; i++) {
        vga_print(" ");
    }

    if (status == 0) {
        vga_set_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
        vga_print("[ OK ]\n");
    } else if (status == 1) {
        vga_set_color(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
        vga_print("[ WARN ]\n");
    } else {
        vga_set_color(VGA_COLOR_LIGHT_RED, VGA_COLOR_BLACK);
        vga_print("[ FAIL ]\n");
    }

    serial_print("\033[1;34m::\033[0m ");
    serial_print(msg);

    padding = 72 - 3 - len;
    if (padding < 1) {
        padding = 1;
    }
    for (int i = 0; i < padding; i++) {
        serial_print(" ");
    }

    if (status == 0) {
        serial_print("\033[1;32m[ OK ]\033[0m\n");
    } else if (status == 1) {
        serial_print("\033[1;33m[WARN]\033[0m\n");
    } else {
        serial_print("\033[1;31m[FAIL]\033[0m\n");
    }
}

/**
 * hw_init - Execute low-level target hardware configuration routines.
 */
void hw_init(void) {
    serial_init();
    vga_init();

    print_boot_log("Initializing Serial Port (COM1) driver", 0);
    print_boot_log("Configuring VGA text-mode frame buffer (80x25)", 0);

    idt_init();
    print_boot_log("Loading Interrupt Descriptor Table (IDT) registers", 0);

    pic_init(32, 40);
    print_boot_log("Remapping dual 8259 PIC interrupt controller IRQs", 0);

    pit_init(1000);
    print_boot_log("Configuring 8253 PIT system timer tick rate to 1000Hz", 0);

    keyboard_init();
    print_boot_log("Initializing PS/2 keyboard controller & driver", 0);

    mouse_init();
    print_boot_log("Initializing PS/2 mouse controller & driver", 0);

    rtc_init();
    print_boot_log("Reading CMOS Real-Time Clock (RTC) date/time registers", 0);

    heap_init(&__heap_start, (size_t)((uintptr_t)&__heap_end - (uintptr_t)&__heap_start));
    print_boot_log("Determining kernel C heap memory boundaries", 0);
    print_boot_log("Initializing local C heap allocator space (1MB)", 0);

    print_boot_log("Completing low-level hardware subsystem checks", 0);
    print_boot_log("Jumping to Rust 64-bit kernel_main() entry point", 0);
}
