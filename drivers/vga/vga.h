/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_VGA_VGA_H
#define KEIRA_DRIVERS_VGA_VGA_H

#include <stdint.h>

/**
 * VGA 80x25 Text Mode Driver Interface
 */

typedef enum {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_YELLOW = 14,
    VGA_COLOR_WHITE = 15,
} vga_color_t;

/**
 * vga_init - Initialize VGA display buffer and clear frame memory.
 */
void vga_init(void);

/**
 * vga_set_color - Set foreground and background text color attributes.
 * @fg: Foreground color selector (0-15).
 * @bg: Background color selector (0-15).
 */
void vga_set_color(uint8_t fg, uint8_t bg);

/**
 * vga_putchar - Render a single ASCII character onto VGA buffer.
 * @c: ASCII character byte to output.
 */
void vga_putchar(char c);

/**
 * vga_print - Render a null-terminated string onto VGA display.
 * @str: Pointer to null-terminated string.
 */
void vga_print(const char *str);

/**
 * vga_print_n - Render a batched string payload onto VGA display with single I/O cursor update.
 * @str: Pointer to string buffer.
 * @len: Length of string in bytes.
 */
void vga_print_n(const char *str, uint64_t len);

/**
 * vga_backspace - Handle backspace input by erasing character before cursor.
 */
void vga_backspace(void);

/**
 * vga_draw_mouse - Render mouse pointer onto text mode screen.
 * @x: X cell coordinate.
 * @y: Y cell coordinate.
 */
void vga_draw_mouse(uint16_t x, uint16_t y);
void vga_draw_mouse_text(uint16_t x, uint16_t y);

/**
 * vga_clear_mouse - Restore background beneath mouse cursor.
 * @x: X cell coordinate.
 * @y: Y cell coordinate.
 */
void vga_clear_mouse(uint16_t x, uint16_t y);
void vga_clear_mouse_text(uint16_t x, uint16_t y);

/**
 * vga_enable_cursor - Configure hardware blinking cursor scanlines.
 * @cursor_start: Top scanline (0-15).
 * @cursor_end: Bottom scanline (0-15).
 */
void vga_enable_cursor(uint8_t cursor_start, uint8_t cursor_end);

/**
 * vga_get_cursor_col - Read active cursor column coordinate.
 *
 * Return: Current column index.
 */
uint16_t vga_get_cursor_col(void);

/**
 * vga_get_cursor_row - Read active cursor row coordinate.
 *
 * Return: Current row index.
 */
uint16_t vga_get_cursor_row(void);

/**
 * vga_set_cursor_pos - Set active cursor row and column coordinates.
 * @row: Row index (0-24).
 * @col: Column index (0-79).
 */
void vga_set_cursor_pos(uint16_t row, uint16_t col);

/**
 * vga_clear_line_from - Clear screen line starting from given column.
 * @col: Starting column index.
 */
void vga_clear_line_from(uint16_t col);

#endif /* KEIRA_DRIVERS_VGA_VGA_H */