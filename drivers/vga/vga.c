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
 * Keira Drivers: VGA Text Console Driver
 *
 * Provides direct 80x25 character display control over VGA video memory (0xB8000).
 * Implements hardware cursor updates, dynamic line wrapping, autoscrolling, and color palettes.
 */

#include "vga.h"

#define VGA_WIDTH 80
#define VGA_HEIGHT 25
#define VGA_MEMORY 0xB8000

static uint16_t *const VGA_BUFFER = (uint16_t *)VGA_MEMORY;
static uint16_t cursor_row = 0;
static uint16_t cursor_col = 0;
static uint8_t current_attr = 0x07;
static uint8_t vga_fb_mode = 0;

/**
 * outb - Write an 8-bit byte value to a port.
 * @port: 16-bit I/O port address.
 * @val: 8-bit value to write.
 */
static inline void outb(uint16_t port, uint8_t val) {
    __asm__ volatile("outb %0, %1" : : "a"(val), "Nd"(port));
}

/**
 * inb - Read an 8-bit byte value from a port.
 * @port: 16-bit I/O port address.
 *
 * Return: 8-bit value read.
 */
static inline uint8_t inb(uint16_t port) {
    uint8_t ret;
    __asm__ volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static uint16_t mouse_cursor_x = 0xFFFF;
static uint16_t mouse_cursor_y = 0xFFFF;
static uint16_t saved_mouse_entry = 0;
static uint8_t mouse_is_visible = 0;

/**
 * vga_set_fb_mode - Configure VGA framebuffer mode state.
 * @mode: Framebuffer mode flag.
 */
void vga_set_fb_mode(uint8_t mode) {
    vga_fb_mode = mode;
}

/**
 * vga_hide_mouse_internal - Erase mouse pointer character cell and restore background.
 */
static void vga_hide_mouse_internal(void) {
    if (mouse_is_visible) {
        if (mouse_cursor_x < VGA_WIDTH && mouse_cursor_y < VGA_HEIGHT) {
            uint16_t index = mouse_cursor_y * VGA_WIDTH + mouse_cursor_x;
            VGA_BUFFER[index] = saved_mouse_entry;
        }
        mouse_is_visible = 0;
    }
}

/**
 * vga_show_mouse_internal - Draw mouse pointer character cell over screen.
 */
static void vga_show_mouse_internal(void) {
    if (!mouse_is_visible && mouse_cursor_x < VGA_WIDTH && mouse_cursor_y < VGA_HEIGHT) {
        uint16_t index = mouse_cursor_y * VGA_WIDTH + mouse_cursor_x;
        uint16_t entry = VGA_BUFFER[index];
        saved_mouse_entry = entry;

        uint8_t bg = (entry >> 12) & 0x0F;
        uint8_t pointer_char = 24;
        uint8_t pointer_attr;

        if (bg == VGA_COLOR_WHITE || bg == VGA_COLOR_LIGHT_GREY || bg == VGA_COLOR_YELLOW ||
            bg == VGA_COLOR_LIGHT_CYAN) {
            pointer_attr = VGA_COLOR_BLACK | (bg << 4);
        } else {
            pointer_attr = VGA_COLOR_WHITE | (bg << 4);
        }

        VGA_BUFFER[index] = (uint16_t)pointer_char | ((uint16_t)pointer_attr << 8);
        mouse_is_visible = 1;
    }
}

/**
 * vga_make_entry - Package ASCII char and color attribute into a 16-bit cell.
 * @c: ASCII character.
 * @attr: Color attribute byte.
 *
 * Return: Structured 16-bit display cell value.
 */
static inline uint16_t vga_make_entry(char c, uint8_t attr) {
    return (uint16_t)c | ((uint16_t)attr << 8);
}

/**
 * vga_make_color - Package foreground and background colors.
 * @fg: Foreground color.
 * @bg: Background color.
 *
 * Return: Structured color attribute byte.
 */
static inline uint8_t vga_make_color(uint8_t fg, uint8_t bg) {
    return fg | (bg << 4);
}

/**
 * vga_scroll - Scroll the screen display upward by one character line.
 */
static void vga_scroll(void) {
    uint64_t *dst = (uint64_t *)VGA_BUFFER;
    uint64_t *src = (uint64_t *)(VGA_BUFFER + VGA_WIDTH);
    for (uint32_t i = 0; i < 480; i++) {
        dst[i] = src[i];
    }

    uint16_t blank = vga_make_entry(' ', current_attr);
    uint32_t last_row_start = (VGA_HEIGHT - 1) * VGA_WIDTH;
    for (uint16_t col = 0; col < VGA_WIDTH; col++) {
        VGA_BUFFER[last_row_start + col] = blank;
    }
}

/**
 * vga_update_cursor - Program hardware VGA register cursor cell position.
 */
static void vga_update_cursor(void) {
    uint16_t pos = cursor_row * VGA_WIDTH + cursor_col;

    outb(0x3D4, 0x0F);
    outb(0x3D5, (uint8_t)(pos & 0xFF));
    outb(0x3D4, 0x0E);
    outb(0x3D5, (uint8_t)((pos >> 8) & 0xFF));
}

/**
 * vga_enable_cursor - Configure hardware blinking cursor scanlines.
 * @cursor_start: Top scanline (0-15).
 * @cursor_end: Bottom scanline (0-15).
 */
void vga_enable_cursor(uint8_t cursor_start, uint8_t cursor_end) {
    outb(0x3D4, 0x0A);
    outb(0x3D5, (inb(0x3D5) & 0xC0) | cursor_start);

    outb(0x3D4, 0x0B);
    outb(0x3D5, (inb(0x3D5) & 0xE0) | cursor_end);
}

/**
 * vga_init - Initialize VGA display buffer and clear frame memory.
 */
void vga_init(void) {
    if (vga_fb_mode) {
        return;
    }
    vga_hide_mouse_internal();
    cursor_row = 0;
    cursor_col = 0;

    uint16_t blank = vga_make_entry(' ', current_attr);
    for (uint16_t i = 0; i < VGA_WIDTH * VGA_HEIGHT; i++) {
        VGA_BUFFER[i] = blank;
    }

    vga_enable_cursor(13, 15);
    vga_update_cursor();
    vga_show_mouse_internal();
}

/**
 * vga_set_color - Set foreground and background text color attributes.
 * @fg: Foreground color selector (0-15).
 * @bg: Background color selector (0-15).
 */
void vga_set_color(uint8_t fg, uint8_t bg) {
    if (vga_fb_mode) {
        return;
    }
    current_attr = vga_make_color(fg, bg);
}

extern void serial_putchar(char c);

/**
 * vga_putchar - Render a single ASCII character onto VGA buffer.
 * @c: ASCII character byte to output.
 */
void vga_putchar(char c) {
    if (vga_fb_mode) {
        return;
    }
    vga_hide_mouse_internal();

    if (c == '\n') {
        cursor_col = 0;
        cursor_row++;
    } else if (c == '\r') {
        cursor_col = 0;
    } else {
        uint16_t index = cursor_row * VGA_WIDTH + cursor_col;
        VGA_BUFFER[index] = vga_make_entry(c, current_attr);
        cursor_col++;

        if (cursor_col >= VGA_WIDTH) {
            cursor_col = 0;
            cursor_row++;
        }
    }

    while (cursor_row >= VGA_HEIGHT) {
        vga_scroll();
        cursor_row--;
    }

    vga_update_cursor();
    vga_show_mouse_internal();
}

/**
 * vga_print_n - Render a batched string payload onto VGA display with single I/O cursor update.
 * @str: Pointer to string buffer.
 * @len: Length of string in bytes.
 */
void vga_print_n(const char *str, uint64_t len) {
    if (vga_fb_mode || !str || len == 0) {
        return;
    }
    vga_hide_mouse_internal();

    for (uint64_t i = 0; i < len; i++) {
        char c = str[i];
        if (c == '\n') {
            cursor_col = 0;
            cursor_row++;
        } else if (c == '\r') {
            cursor_col = 0;
        } else {
            uint16_t index = cursor_row * VGA_WIDTH + cursor_col;
            VGA_BUFFER[index] = vga_make_entry(c, current_attr);
            cursor_col++;

            if (cursor_col >= VGA_WIDTH) {
                cursor_col = 0;
                cursor_row++;
            }
        }

        while (cursor_row >= VGA_HEIGHT) {
            vga_scroll();
            cursor_row--;
        }
    }

    vga_update_cursor();
    vga_show_mouse_internal();
}

/**
 * vga_print - Render a null-terminated string onto VGA display.
 * @str: Pointer to null-terminated string.
 */
void vga_print(const char *str) {
    if (!str) {
        return;
    }
    uint64_t len = 0;
    while (str[len] != '\0') {
        len++;
    }
    vga_print_n(str, len);
}

/**
 * vga_backspace - Handle backspace input by erasing character before cursor.
 */
void vga_backspace(void) {
    if (vga_fb_mode) {
        return;
    }
    vga_hide_mouse_internal();
    if (cursor_col == 0) {
        if (cursor_row > 0) {
            cursor_row--;
            cursor_col = VGA_WIDTH - 1;
        }
    } else {
        cursor_col--;
    }

    uint16_t index = cursor_row * VGA_WIDTH + cursor_col;
    VGA_BUFFER[index] = vga_make_entry(' ', current_attr);
    vga_update_cursor();
    vga_show_mouse_internal();
}

/**
 * vga_draw_mouse_text - Render mouse pointer onto text mode screen.
 * @x: X cell coordinate.
 * @y: Y cell coordinate.
 */
void vga_draw_mouse_text(uint16_t x, uint16_t y) {
    vga_hide_mouse_internal();
    mouse_cursor_x = x;
    mouse_cursor_y = y;
    vga_show_mouse_internal();
}

/**
 * vga_clear_mouse_text - Restore background beneath mouse cursor.
 * @x: X cell coordinate.
 * @y: Y cell coordinate.
 */
void vga_clear_mouse_text(uint16_t x, uint16_t y) {
    (void)x;
    (void)y;
    vga_hide_mouse_internal();
}

/**
 * vga_get_cursor_col - Read active cursor column coordinate.
 *
 * Return: Current column index.
 */
uint16_t vga_get_cursor_col(void) {
    return cursor_col;
}

/**
 * vga_get_cursor_row - Read active cursor row coordinate.
 *
 * Return: Current row index.
 */
uint16_t vga_get_cursor_row(void) {
    return cursor_row;
}

/**
 * vga_set_cursor_pos - Set active cursor row and column coordinates.
 * @row: Row index (0-24).
 * @col: Column index (0-79).
 */
void vga_set_cursor_pos(uint16_t row, uint16_t col) {
    if (vga_fb_mode) {
        return;
    }
    vga_hide_mouse_internal();
    while (row >= VGA_HEIGHT) {
        vga_scroll();
        row--;
    }
    cursor_row = row;
    cursor_col = col;
    vga_update_cursor();
    vga_show_mouse_internal();
}

/**
 * vga_clear_line_from - Clear screen line starting from given column.
 * @col: Starting column index.
 */
void vga_clear_line_from(uint16_t col) {
    if (vga_fb_mode) {
        return;
    }
    vga_hide_mouse_internal();
    uint16_t blank = vga_make_entry(' ', current_attr);
    for (uint16_t c = col; c < VGA_WIDTH; c++) {
        uint16_t index = cursor_row * VGA_WIDTH + c;
        VGA_BUFFER[index] = blank;
    }
    vga_show_mouse_internal();
}