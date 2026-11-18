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

//! Emergency diagnostic and help utilities.

pub mod misc {
    pub mod help {
        use keira_io::vga;

        pub fn run(_parts: &mut core::str::SplitWhitespace) {
            let bg = vga::Color::Black;
            let fg = vga::Color::White;

            vga::set_color(fg, bg);
            vga::print_str(concat!(
                "Keira Kernel Diagnostic Console (Ring 0 v",
                env!("CARGO_PKG_VERSION"),
                ")\n\n"
            ));
            vga::print_str(
                "Diagnostics : system   cpu      smp      memory   slabinfo  regs     irq\n",
            );
            vga::print_str("Hardware    : devices  drivers  disk     dcache   icache    pipes\n");
            vga::print_str("Security    : secinfo  aslrinfo rnginfo  rlimit   unixsock  vmainfo\n");
            vga::print_str("Multiplex   : pollinfo selectinfo\n");
            vga::print_str(
                "Control     : syslog   wipe     sync     unwind   sh        run <path>\n",
            );
            vga::print_str("Lifecycle   : reset    power\n\n");
            vga::print_str("Type '<cmd> --help' for options. Press Tab for completion.\n");
        }
    }
    pub use self::help::*;
}

pub use self::misc::help;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util_commands_help_invocation() {
        let mut args = "help --help".split_whitespace();
        args.next();
        help::run(&mut args);
    }
}
