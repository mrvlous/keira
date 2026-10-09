// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interactive terminal main loop and CPU idle dispatcher.

use keira_io::vga;
use keira_shell as shell;

/// Launch interactive terminal session and enter the primary kernel event loop.
pub fn enter_main_loop() -> ! {
    // Attempt to launch canonical userspace init (PID 1) in Ring 3 if present
    unsafe {
        if keira_fs::exists("/bin/init.elf") || keira_fs::exists("/bin/init") {
            let init_path = if keira_fs::exists("/bin/init.elf") {
                "/bin/init.elf"
            } else {
                "/bin/init"
            };

            let _ = shell::run_user_program(init_path, &[init_path]);
        }
    }

    vga::init();

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("-keira-1 (tty1)\n\n");

    #[cfg(target_os = "none")]
    unsafe {
        core::arch::asm!("sti");
    }

    shell::run_boot_script();
    shell::print_prompt();

    loop {
        shell::process_pending();
        #[cfg(target_os = "none")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
