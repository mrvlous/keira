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
    // Run canonical early userspace init (PID 1) in Ring 3 if present
    unsafe {
        let init_path = if keira_fs::exists("/bin/init.elf") {
            Some("/bin/init.elf")
        } else if keira_fs::exists("/bin/init") {
            Some("/bin/init")
        } else {
            None
        };

        if let Some(path) = init_path {
            let _ = shell::run_user_program(path, &[path]);
        }
    }

    vga::init();

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    #[cfg(target_arch = "x86_64")]
    vga::print_str(" (x86_64) - tty1\n\n");
    #[cfg(target_arch = "x86")]
    vga::print_str(" (i686) - tty1\n\n");

    #[cfg(target_os = "none")]
    unsafe {
        core::arch::asm!("sti");
    }

    shell::run_boot_script();

    // Direct handover to canonical Ring 3 userspace shell on 64-bit systems
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let sh_path = if keira_fs::exists("/bin/sh.elf") {
            Some("/bin/sh.elf")
        } else if keira_fs::exists("/bin/sh") {
            Some("/bin/sh")
        } else {
            None
        };

        if let Some(path) = sh_path {
            let _ = shell::run_user_program(path, &[path]);
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("\n[kernel] Userspace session terminated.\n");
            vga::print_str(
                "[kernel] Dropping to supervisor console (Ring 0). Type 'help' or 'sh'.\n\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        } else {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str(
                "[kernel] Pure freestanding Ring 0 mode: No userland /bin/sh attached.\n",
            );
            vga::print_str(
                "[kernel] Entering supervisor console (Ring 0). Type 'help' for diagnostics.\n\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }

    #[cfg(target_arch = "x86")]
    if !keira_fs::exists("/bin/sh.elf") && !keira_fs::exists("/bin/sh") {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[kernel] Pure freestanding Ring 0 mode: No userland /bin/sh attached.\n");
        vga::print_str(
            "[kernel] Entering supervisor console (Ring 0). Type 'help' for diagnostics.\n\n",
        );
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }

    shell::print_prompt();

    loop {
        shell::process_pending();
        #[cfg(target_os = "none")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
