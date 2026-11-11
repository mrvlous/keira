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

//! Implementation of the 'unixsock' and 'unix' shell commands.

use crate::args::CliArgs;
use keira_io::vga;
use keira_net::socket::{for_each_unix_socket, SocketEntry, SOCKET_BUF_SIZE};

fn print_padded_str(s: &str, width: usize) {
    vga::print_str(s);
    let len = s.len();
    for _ in len..width {
        vga::print_str(" ");
    }
}

fn print_num(val: u64, width: usize) {
    let mut digits = 1;
    let mut temp = val;
    while temp >= 10 {
        digits += 1;
        temp /= 10;
    }
    vga::print_u64(val);
    for _ in digits..width {
        vga::print_str(" ");
    }
}

/// Executes the 'unixsock' / 'unix' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: unixsock [-v] [-h]\n\n");
        vga::print_str(
            "Description:\n  Display active UNIX domain sockets (AF_UNIX) and inter-process IPC channels.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -v, --verbose  Display buffer capacities and connected peer endpoints\n");
        vga::print_str("  -h, --help     Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let is_verbose = args.has_flag('v', "verbose");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("=== Active UNIX Domain Sockets (AF_UNIX IPC Table) ===\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    if is_verbose {
        vga::print_str("ID   Type     State        Peer   RX Queue     TX Queue     Path / Role\n");
        vga::print_str(
            "---- -------- ------------ ------ ------------ ------------ --------------------\n",
        );
    } else {
        vga::print_str("ID   Type     State        Peer   RX Queue     Path / Role\n");
        vga::print_str("---- -------- ------------ ------ ------------ --------------------\n");
    }

    let mut count = 0usize;
    unsafe {
        for_each_unix_socket(|sock: &SocketEntry| {
            count += 1;
            print_num(sock.id as u64, 5);

            let type_str = if sock.socket_type == 1 {
                "STREAM  "
            } else if sock.socket_type == 2 {
                "DGRAM   "
            } else {
                "RAW     "
            };
            vga::print_str(type_str);

            let state_str = if sock.is_connected {
                "CONNECTED   "
            } else if sock.path_len > 0 {
                "LISTENING   "
            } else {
                "UNBOUND     "
            };
            vga::print_str(state_str);

            if sock.peer_id != 0 {
                print_num(sock.peer_id as u64, 7);
            } else {
                print_padded_str("-", 7);
            }

            vga::print_u64(sock.rx_len as u64);
            vga::print_str("/");
            vga::print_u64(SOCKET_BUF_SIZE as u64);
            vga::print_str("B    ");

            if is_verbose {
                vga::print_u64(sock.tx_len as u64);
                vga::print_str("/");
                vga::print_u64(SOCKET_BUF_SIZE as u64);
                vga::print_str("B    ");
            }

            if sock.path_len > 0 {
                if let Ok(p) = core::str::from_utf8(&sock.path[..sock.path_len]) {
                    vga::print_str(p);
                } else {
                    vga::print_str("[invalid path]");
                }
            } else if sock.peer_id != 0 {
                vga::print_str("[socketpair]");
            } else {
                vga::print_str("[anonymous]");
            }
            vga::print_str("\n");
        });
    }

    if count == 0 {
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        vga::print_str("(No active UNIX domain sockets allocated)\n");
    }

    vga::print_str("\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
}
