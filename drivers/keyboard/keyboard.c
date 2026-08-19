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
 * PS/2 Keyboard Peripheral Driver Implementation
 */

#include "keyboard.h"

#include "scancodes.h"

#include <asm/io.h>
#include <asm/pic.h>

#define KBD_DATA_PORT 0x60

extern void shell_handle_keypress(uint8_t c);

static int shift_pressed = 0;
static int ctrl_pressed = 0;

/**
 * keyboard_init - Unmask keyboard IRQ line on PIC.
 */
void keyboard_init(void) {
    pic_clear_mask(1);
}

/**
 * keyboard_handler - Read scan codes from data port 0x60 and dispatch events.
 */
void keyboard_handler(void) {
    uint8_t scancode = inb(KBD_DATA_PORT);

    if (scancode & 0x80) {
        if (scancode == (KEY_LSHIFT | 0x80) || scancode == (KEY_RSHIFT | 0x80)) {
            shift_pressed = 0;
        } else if (scancode == (KEY_LCTRL | 0x80)) {
            ctrl_pressed = 0;
        }
    } else {
        if (scancode == KEY_LSHIFT || scancode == KEY_RSHIFT) {
            shift_pressed = 1;
        } else if (scancode == KEY_LCTRL) {
            ctrl_pressed = 1;
        } else if (scancode == KEY_UP) {
            shell_handle_keypress(0x80);
        } else if (scancode == KEY_DOWN) {
            shell_handle_keypress(0x81);
        } else if (scancode == KEY_LEFT) {
            shell_handle_keypress(0x82);
        } else if (scancode == KEY_RIGHT) {
            shell_handle_keypress(0x83);
        } else if (scancode == KEY_F3) {
            shell_handle_keypress(0x84);
        } else if (scancode == KEY_F10) {
            shell_handle_keypress(0x85);
        } else {
            if (scancode < 128) {
                unsigned char c =
                    shift_pressed ? kbd_us_shifted_layout[scancode] : kbd_us_layout[scancode];

                if (c != 0) {
                    if (ctrl_pressed) {
                        if (c >= 'a' && c <= 'z') {
                            shell_handle_keypress(c - 'a' + 1);
                            pic_eoi(1);
                            return;
                        }
                        if (c >= 'A' && c <= 'Z') {
                            shell_handle_keypress(c - 'A' + 1);
                            pic_eoi(1);
                            return;
                        }
                    }
                    shell_handle_keypress(c);
                }
            }
        }
    }

    pic_eoi(1);
}