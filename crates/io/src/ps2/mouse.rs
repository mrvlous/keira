// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Auxiliary Mouse Controller Driver in pure Rust.

use crate::vga::console::{vga_clear_mouse, vga_draw_mouse};
use keira_arch::cpu::{inb, outb};
use keira_arch::interrupts::pic;

pub const PS2_DATA_PORT: u16 = 0x60;
pub const PS2_STATUS_PORT: u16 = 0x64;
pub const PS2_COMMAND_PORT: u16 = 0x64;

const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;
const PS2_STATUS_INPUT_FULL: u8 = 0x02;
const PS2_STATUS_MOUSE_DATA: u8 = 0x20;

const PS2_CMD_READ_CONFIG: u8 = 0x20;
const PS2_CMD_WRITE_CONFIG: u8 = 0x60;
const PS2_CMD_ENABLE_MOUSE: u8 = 0xA8;
const PS2_CMD_WRITE_MOUSE: u8 = 0xD4;

const MOUSE_CMD_SET_DEFAULTS: u8 = 0xF6;
const MOUSE_CMD_ENABLE_STREAM: u8 = 0xF4;

const MOUSE_FLAGS_SYNC: u8 = 0x08;
const MOUSE_FLAGS_X_OVERFLOW: u8 = 0x40;
const MOUSE_FLAGS_Y_OVERFLOW: u8 = 0x80;

const MOUSE_IRQ: u8 = 12;

static mut MOUSE_CYCLE: u8 = 0;
static mut MOUSE_BYTE: [i8; 3] = [0; 3];

static mut MOUSE_X: i32 = 40;
static mut MOUSE_Y: i32 = 12;
static mut MOUSE_FX: i32 = 40 * 256;
static mut MOUSE_FY: i32 = 12 * 256;

static mut MOUSE_MAX_X: i32 = 80;
static mut MOUSE_MAX_Y: i32 = 25;
static mut MOUSE_SENSITIVITY_X: i32 = 24;
static mut MOUSE_SENSITIVITY_Y: i32 = 12;

/// Set coordinate boundaries and sensitivity based on screen resolution.
pub fn set_resolution(width: i32, height: i32) {
    unsafe {
        MOUSE_MAX_X = width;
        MOUSE_MAX_Y = height;
        MOUSE_X = width / 2;
        MOUSE_Y = height / 2;
        MOUSE_FX = MOUSE_X * 256;
        MOUSE_FY = MOUSE_Y * 256;

        if width > 80 {
            MOUSE_SENSITIVITY_X = 512;
            MOUSE_SENSITIVITY_Y = 512;
        } else {
            MOUSE_SENSITIVITY_X = 24;
            MOUSE_SENSITIVITY_Y = 12;
        }
    }
}

fn mouse_wait(a_type: u8) {
    let mut timeout: u32 = 100_000;
    unsafe {
        if a_type == 0 {
            while timeout > 0 {
                timeout -= 1;
                if (inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) == PS2_STATUS_OUTPUT_FULL {
                    return;
                }
            }
        } else {
            while timeout > 0 {
                timeout -= 1;
                if (inb(PS2_STATUS_PORT) & PS2_STATUS_INPUT_FULL) == 0 {
                    return;
                }
            }
        }
    }
}

fn mouse_write(cmd: u8) {
    mouse_wait(1);
    unsafe {
        outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_MOUSE);
    }
    mouse_wait(1);
    unsafe {
        outb(PS2_DATA_PORT, cmd);
    }
}

fn mouse_read() -> u8 {
    mouse_wait(0);
    unsafe { inb(PS2_DATA_PORT) }
}

/// Initialize PS/2 auxiliary mouse controller and unmask IRQ12.
pub fn init() {
    mouse_wait(1);
    unsafe {
        outb(PS2_COMMAND_PORT, PS2_CMD_ENABLE_MOUSE);
    }

    mouse_wait(1);
    unsafe {
        outb(PS2_COMMAND_PORT, PS2_CMD_READ_CONFIG);
    }
    mouse_wait(0);
    let mut status = unsafe { inb(PS2_DATA_PORT) };

    status |= 2;
    status &= !0x20;

    mouse_wait(1);
    unsafe {
        outb(PS2_COMMAND_PORT, PS2_CMD_WRITE_CONFIG);
    }
    mouse_wait(1);
    unsafe {
        outb(PS2_DATA_PORT, status);
    }

    mouse_write(MOUSE_CMD_SET_DEFAULTS);
    let _ = mouse_read();

    mouse_write(MOUSE_CMD_ENABLE_STREAM);
    let _ = mouse_read();

    pic::clear_mask(MOUSE_IRQ);

    unsafe {
        vga_draw_mouse(MOUSE_X as u16, MOUSE_Y as u16);
    }
}

/// IRQ12 PS/2 mouse interrupt service routine handler.
#[no_mangle]
pub extern "C" fn mouse_handler() {
    unsafe {
        let status = inb(PS2_STATUS_PORT);

        if (status & PS2_STATUS_MOUSE_DATA) == PS2_STATUS_MOUSE_DATA
            && (status & PS2_STATUS_OUTPUT_FULL) == PS2_STATUS_OUTPUT_FULL
        {
            let byte = inb(PS2_DATA_PORT) as i8;
            MOUSE_BYTE[MOUSE_CYCLE as usize] = byte;
            MOUSE_CYCLE += 1;

            if MOUSE_CYCLE == 1 && ((MOUSE_BYTE[0] as u8) & MOUSE_FLAGS_SYNC) == 0 {
                MOUSE_CYCLE = 0;
                pic::send_eoi(MOUSE_IRQ);
                return;
            }

            if MOUSE_CYCLE == 3 {
                MOUSE_CYCLE = 0;

                let flags = MOUSE_BYTE[0] as u8;
                if (flags & (MOUSE_FLAGS_X_OVERFLOW | MOUSE_FLAGS_Y_OVERFLOW)) == 0 {
                    MOUSE_FX += (MOUSE_BYTE[1] as i32) * MOUSE_SENSITIVITY_X;
                    MOUSE_FY -= (MOUSE_BYTE[2] as i32) * MOUSE_SENSITIVITY_Y;

                    if MOUSE_FX < 0 {
                        MOUSE_FX = 0;
                    }
                    if MOUSE_FX >= MOUSE_MAX_X * 256 {
                        MOUSE_FX = (MOUSE_MAX_X - 1) * 256;
                    }
                    if MOUSE_FY < 0 {
                        MOUSE_FY = 0;
                    }
                    if MOUSE_FY >= MOUSE_MAX_Y * 256 {
                        MOUSE_FY = (MOUSE_MAX_Y - 1) * 256;
                    }

                    let new_x = MOUSE_FX / 256;
                    let new_y = MOUSE_FY / 256;

                    if new_x != MOUSE_X || new_y != MOUSE_Y {
                        vga_clear_mouse(MOUSE_X as u16, MOUSE_Y as u16);
                        MOUSE_X = new_x;
                        MOUSE_Y = new_y;
                        vga_draw_mouse(MOUSE_X as u16, MOUSE_Y as u16);
                    }
                }
            }
        } else if (status & PS2_STATUS_OUTPUT_FULL) != 0 {
            inb(PS2_DATA_PORT);
        }

        pic::send_eoi(MOUSE_IRQ);
    }
}

// C-compatible export for resolution changes
#[no_mangle]
pub extern "C" fn mouse_set_resolution(width: i32, height: i32) {
    set_resolution(width, height);
}
