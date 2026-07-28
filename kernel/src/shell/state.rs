// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Terminal Shell and Editor Global States

use crate::io::vga;

pub const BUFFER_SIZE: usize = 256;
pub static mut INPUT_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
pub static mut BUFFER_LEN: usize = 0;
pub static mut COMMAND_READY: bool = false;

// Command history ring buffer
pub const HISTORY_SIZE: usize = 16;
pub static mut HISTORY: [[u8; BUFFER_SIZE]; HISTORY_SIZE] = [[0; BUFFER_SIZE]; HISTORY_SIZE];
pub static mut HISTORY_LENS: [usize; HISTORY_SIZE] = [0; HISTORY_SIZE];
// Total commands entered
pub static mut HISTORY_COUNT: usize = 0;
// Current browsing index (-1 = not browsing)
pub static mut HISTORY_INDEX: isize = -1;

// Prompt length for clearing input on history navigation
pub static mut PROMPT_COL: u16 = 0;
pub static mut PROMPT_ROW: u16 = 0;

// Editor state variables
pub static mut IN_EDITOR_MODE: bool = false;
pub static mut EDITOR_GRID: [[u8; 80]; 23] = [[b' '; 80]; 23];
pub static mut LINE_LENS: [u16; 23] = [0; 23];
pub static mut EDIT_FILENAME: [u8; 12] = [0; 12];
pub static mut EDIT_FILENAME_LEN: usize = 0;
pub static mut EDIT_CUR_X: u16 = 0;
pub static mut EDIT_CUR_Y: u16 = 0;
pub static mut EDITOR_CONFIRM_SAVE: bool = false;
pub static mut EDITOR_CONFIRM_EXIT: bool = false;
pub static mut EDITOR_STATUS_MSG: [u8; 40] = [0; 40];
pub static mut EDITOR_STATUS_LEN: usize = 0;
pub static mut EDITOR_STATUS_COLOR: vga::Color = vga::Color::LightGreen;

pub static mut SHELL_PATH: [u8; 80] = [0u8; 80];
pub static mut SHELL_PATH_LEN: usize = 0;

#[derive(Copy, Clone)]
pub struct ShellTheme {
    pub user: vga::Color,
    pub host: vga::Color,
    pub path: vga::Color,
    pub symbol: vga::Color,
    pub text_fg: vga::Color,
    pub text_bg: vga::Color,
}

pub static mut CURRENT_THEME: ShellTheme = ShellTheme {
    user: vga::Color::LightRed,
    host: vga::Color::LightCyan,
    path: vga::Color::LightBlue,
    symbol: vga::Color::LightGreen,
    text_fg: vga::Color::LightGrey,
    text_bg: vga::Color::Black,
};

// Please and User Account Management States
pub static mut IN_PLEASE_MODE: bool = false;
pub static mut PLEASE_COMMAND: [u8; 128] = [0; 128];
pub static mut PLEASE_COMMAND_LEN: usize = 0;
pub static mut IN_LOGIN_MODE: bool = false;
pub static mut LOGIN_USERNAME: [u8; 16] = [0; 16];
pub static mut LOGIN_USERNAME_LEN: usize = 0;
pub static mut CURRENT_USER: [u8; 16] = *b"default         ";
pub static mut CURRENT_USER_LEN: usize = 7;
pub static mut IS_ADMIN: bool = false;

// Editor Search Mode States
pub static mut IN_SEARCH_MODE: bool = false;
pub static mut SEARCH_BUFFER: [u8; 16] = [0; 16];
pub static mut SEARCH_LEN: usize = 0;
