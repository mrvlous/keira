// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! PS/2 Mouse packet flags, commands, and coordinate sensitivity factors.

/// PS/2 Status Register flag: Output buffer full.
pub const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;

/// PS/2 Status Register flag: Input buffer full.
pub const PS2_STATUS_INPUT_FULL: u8 = 0x02;

/// PS/2 Status Register flag: Auxiliary device (mouse) output data present.
pub const PS2_STATUS_MOUSE_DATA: u8 = 0x20;

/// PS/2 Controller command: Read configuration byte.
pub const PS2_CMD_READ_CONFIG: u8 = 0x20;

/// PS/2 Controller command: Write configuration byte.
pub const PS2_CMD_WRITE_CONFIG: u8 = 0x60;

/// PS/2 Controller command: Enable auxiliary mouse interface.
pub const PS2_CMD_ENABLE_MOUSE: u8 = 0xA8;

/// PS/2 Controller command: Write to auxiliary mouse interface.
pub const PS2_CMD_WRITE_MOUSE: u8 = 0xD4;

/// Auxiliary mouse command: Reset to default state.
pub const MOUSE_CMD_SET_DEFAULTS: u8 = 0xF6;

/// Auxiliary mouse command: Enable data reporting stream.
pub const MOUSE_CMD_ENABLE_STREAM: u8 = 0xF4;

/// Mouse packet header flag: Synchronization bit (must be set in byte 0).
pub const MOUSE_FLAGS_SYNC: u8 = 0x08;

/// Mouse packet header flag: Horizontal X overflow.
pub const MOUSE_FLAGS_X_OVERFLOW: u8 = 0x40;

/// Mouse packet header flag: Vertical Y overflow.
pub const MOUSE_FLAGS_Y_OVERFLOW: u8 = 0x80;

/// Master PIC interrupt request line for auxiliary PS/2 mouse.
pub const MOUSE_IRQ: u8 = 12;
