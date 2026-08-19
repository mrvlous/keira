/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_KEYBOARD_SCANCODES_H
#define KEIRA_DRIVERS_KEYBOARD_SCANCODES_H

/**
 * PS/2 Scan Code Set 1 Key Code Definitions
 */

#define KEY_ESCAPE 0x01
#define KEY_BACKSPACE 0x0E
#define KEY_TAB 0x0F
#define KEY_ENTER 0x1C
#define KEY_LCTRL 0x1D
#define KEY_LSHIFT 0x2A
#define KEY_RSHIFT 0x36
#define KEY_LALT 0x38
#define KEY_SPACE 0x39
#define KEY_CAPSLOCK 0x3A

#define KEY_F1 0x3B
#define KEY_F2 0x3C
#define KEY_F3 0x3D
#define KEY_F4 0x3E
#define KEY_F5 0x3F
#define KEY_F6 0x40
#define KEY_F7 0x41
#define KEY_F8 0x42
#define KEY_F9 0x43
#define KEY_F10 0x44

#define KEY_NUMLOCK 0x45
#define KEY_SCROLLLOCK 0x46

#define KEY_HOME 0x47
#define KEY_UP 0x48
#define KEY_PAGEUP 0x49
#define KEY_LEFT 0x4B
#define KEY_RIGHT 0x4D
#define KEY_END 0x4F
#define KEY_DOWN 0x50
#define KEY_PAGEDOWN 0x51
#define KEY_INSERT 0x52
#define KEY_DELETE 0x53

#define KEY_F11 0x57
#define KEY_F12 0x58

/**
 * US QWERTY Scan Code Set 1 to ASCII Lookup Table (Unshifted)
 */
static const char kbd_us_layout[128] = {
    0,   27,  '1', '2', '3',  '4', '5', '6',  '7', '8', '9',  '0', '-', '=', '\b', '\t', 'q', 'w',
    'e', 'r', 't', 'y', 'u',  'i', 'o', 'p',  '[', ']', '\n', 0,   'a', 's', 'd',  'f',  'g', 'h',
    'j', 'k', 'l', ';', '\'', '`', 0,   '\\', 'z', 'x', 'c',  'v', 'b', 'n', 'm',  ',',  '.', '/',
    0,   '*', 0,   ' ', 0,    0,   0,   0,    0,   0,   0,    0,   0,   0,   0,    0,    0,   0,
    0,   0,   '-', 0,   0,    0,   '+', 0,    0,   0,   0,    0,   0,   0,   0,    0,    0,   0};

/**
 * US QWERTY Scan Code Set 1 to ASCII Lookup Table (Shifted)
 */
static const char kbd_us_shifted_layout[128] = {
    0,   27,  '!', '@', '#', '$', '%', '^', '&', '*', '(',  ')', '_', '+', '\b', '\t', 'Q', 'W',
    'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', 0,   'A', 'S', 'D',  'F',  'G', 'H',
    'J', 'K', 'L', ':', '"', '~', 0,   '|', 'Z', 'X', 'C',  'V', 'B', 'N', 'M',  '<',  '>', '?',
    0,   '*', 0,   ' ', 0,   0,   0,   0,   0,   0,   0,    0,   0,   0,   0,    0,    0,   0,
    0,   0,   '-', 0,   0,   0,   '+', 0,   0,   0,   0,    0,   0,   0,   0,    0,    0,   0};

#endif /* KEIRA_DRIVERS_KEYBOARD_SCANCODES_H */