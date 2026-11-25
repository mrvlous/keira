// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Interactive terminal main loop and CPU idle dispatcher.

use keira_io::vga;
use keira_shell as shell;

/// Launch interactive terminal session and enter the primary kernel event loop.
pub fn enter_main_loop() -> ! {
    vga::init();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    #[cfg(target_arch = "x86_64")]
    vga::print_str(" (x86_64)\n\n");
    #[cfg(target_arch = "x86")]
    vga::print_str(" (i686)\n\n");

    #[cfg(target_os = "none")]
    unsafe {
        core::arch::asm!("sti");
    }

    shell::run_boot_script();

    let userland_requested =
        cfg!(kconfig_userland) && !keira_core::has_boot_cmdline_flag("console=ring0");

    // Launch optional early userspace init (respecting init= override) when enabled
    #[cfg(target_arch = "x86_64")]
    if userland_requested {
        unsafe {
            let mut cmd_init_buf = [0u8; 64];
            let custom_init = keira_core::get_boot_cmdline_value("init", &mut cmd_init_buf)
                .filter(|p| keira_fs::exists(p));

            let init_path = if let Some(p) = custom_init {
                Some(p)
            } else if keira_fs::exists("/bin/init.elf") {
                Some("/bin/init.elf")
            } else if keira_fs::exists("/bin/init") {
                Some("/bin/init")
            } else {
                None
            };

            if let Some(path) = init_path {
                keira_core::pr_info!("init", "executing userspace init binary: {}", path);
                let _ = shell::run_user_program(path, &[path]);
            }

            let sh_path = if keira_fs::exists("/bin/sh.elf") {
                Some("/bin/sh.elf")
            } else if keira_fs::exists("/bin/sh") {
                Some("/bin/sh")
            } else {
                None
            };

            if let Some(path) = sh_path {
                vga::set_color(vga::Color::White, vga::Color::Black);
                let _ = shell::run_user_program(path, &[path]);
            } else if init_path.is_none() {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str(
                    "[kernel] Pure freestanding Ring 0 mode: No userland binary attached.\n",
                );
                vga::print_str(
                    "[kernel] Entering supervisor console (Ring 0). Type 'help' for diagnostics.\n\n",
                );
                vga::set_color(vga::Color::White, vga::Color::Black);
            }
        }
    } else {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[kernel] Pure freestanding Ring 0 mode (CONFIG_USERLAND=n).\n");
        vga::print_str(
            "[kernel] Entering supervisor console (Ring 0). Type 'help' for diagnostics.\n\n",
        );
        vga::set_color(vga::Color::White, vga::Color::Black);
    }

    #[cfg(target_arch = "x86")]
    if !userland_requested || (!keira_fs::exists("/bin/sh.elf") && !keira_fs::exists("/bin/sh")) {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[kernel] Pure freestanding Ring 0 mode: No userland /bin/sh attached.\n");
        vga::print_str(
            "[kernel] Entering supervisor console (Ring 0). Type 'help' for diagnostics.\n\n",
        );
        vga::set_color(vga::Color::White, vga::Color::Black);
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

pub mod loop_runner {
    pub use super::enter_main_loop;
}
