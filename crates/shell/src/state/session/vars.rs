// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global shell session variables, screen buffers, and editor state tables.

use keira_io::vga;

pub const BUFFER_SIZE: usize = 256;
pub static mut INPUT_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
pub static mut BUFFER_LEN: usize = 0;
pub static mut COMMAND_READY: bool = false;

pub const HISTORY_SIZE: usize = 16;
pub static mut HISTORY: [[u8; BUFFER_SIZE]; HISTORY_SIZE] = [[0; BUFFER_SIZE]; HISTORY_SIZE];
pub static mut HISTORY_LENS: [usize; HISTORY_SIZE] = [0; HISTORY_SIZE];
pub static mut HISTORY_COUNT: usize = 0;
pub static mut HISTORY_INDEX: isize = -1;

pub static mut PROMPT_COL: u16 = 0;
pub static mut PROMPT_ROW: u16 = 0;

pub static mut IN_EDITOR_MODE: bool = false;
pub static mut EDITOR_GRID: [[u8; 256]; 128] = [[b' '; 256]; 128];
pub static mut LINE_LENS: [u16; 128] = [0; 128];
pub static mut EDIT_FILENAME: [u8; 64] = [0; 64];
pub static mut EDIT_FILENAME_LEN: usize = 0;
pub static mut EDIT_CUR_X: u16 = 0;
pub static mut EDIT_CUR_Y: u16 = 0;
pub static mut EDIT_SCROLL_Y: u16 = 0;
pub static mut EDITOR_MODIFIED: bool = false;
pub static mut EDITOR_CONFIRM_SAVE: bool = false;
pub static mut EDITOR_CONFIRM_EXIT: bool = false;
pub static mut EDITOR_HELP_MODE: bool = false;
pub static mut EDITOR_CUT_BUFFER: [u8; 256] = [b' '; 256];
pub static mut EDITOR_CUT_LEN: u16 = 0;
pub static mut EDITOR_HAS_CUT: bool = false;
pub static mut EDITOR_STATUS_MSG: [u8; 256] = [0; 256];
pub static mut EDITOR_STATUS_LEN: usize = 0;
pub static mut EDITOR_STATUS_COLOR: vga::Color = vga::Color::LightGreen;
pub static mut EDITOR_FILE_BUF: [u8; 16384] = [0; 16384];
pub static mut EDITOR_SCREEN_CHARS: [[u8; 160]; 64] = [[b' '; 160]; 64];
pub static mut EDITOR_SCREEN_FG: [[vga::Color; 160]; 64] = [[vga::Color::LightGrey; 160]; 64];
pub static mut EDITOR_SCREEN_BG: [[vga::Color; 160]; 64] = [[vga::Color::Black; 160]; 64];
pub static mut IN_SEARCH_MODE: bool = false;
pub static mut SEARCH_BUFFER: [u8; 16] = [0; 16];
pub static mut SEARCH_LEN: usize = 0;

pub static mut SHELL_PATH: [u8; 80] = [0u8; 80];
pub static mut SHELL_PATH_LEN: usize = 0;

pub static mut IN_PLEASE_MODE: bool = false;
pub static mut PLEASE_COMMAND: [u8; 128] = [0; 128];
pub static mut PLEASE_COMMAND_LEN: usize = 0;
pub static mut PLEASE_ATTEMPTS: usize = 0;
pub static mut IN_LOGIN_MODE: bool = false;
pub static mut LOGIN_USERNAME: [u8; 16] = [0; 16];
pub static mut LOGIN_USERNAME_LEN: usize = 0;
pub static mut LOGIN_ATTEMPTS: usize = 0;
pub static mut CURRENT_USER: [u8; 16] = *b"admin           ";
pub static mut CURRENT_USER_LEN: usize = 5;
pub static mut IS_ADMIN: bool = false;

pub static mut HOSTNAME: [u8; 32] = *b"keira                           ";
pub static mut HOSTNAME_LEN: usize = 5;

/// Executes a closure under interrupt-safe spinlock protection.
pub fn with_spin_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(all(target_os = "none", not(test)))]
    unsafe {
        core::arch::asm!("cli");
    }
    let res = f();
    #[cfg(all(target_os = "none", not(test)))]
    unsafe {
        core::arch::asm!("sti");
    }
    res
}
