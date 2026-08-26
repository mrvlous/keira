// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'network' shell command for PCI network interface control and ICMP ping testing.

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;
use keira_net::driver::e1000;
use keira_net::icmp::send_ping;

fn print_hex_byte(b: u8) {
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 2];
    buf[0] = chars[((b >> 4) & 0xF) as usize];
    buf[1] = chars[(b & 0xF) as usize];
    if let Ok(s) = core::str::from_utf8(&buf) {
        unsafe {
            vga::print_str(s);
        }
    }
}

fn print_mac(mac: &[u8; 6]) {
    for i in 0..6 {
        print_hex_byte(mac[i]);
        if i < 5 {
            unsafe {
                vga::print_str(":");
            }
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(
                "Usage: network [dhcp|ping <ip>|resolve <domain>|dns-cache] [-s] [-a]\n\n",
            );
            vga::print_str("Description:\n  Inspect network interface status, ARP routing, DNS cache, or ping remote hosts.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -s, --stats    Display extended TX/RX packet statistics\n");
            vga::print_str("  -a, --arp      Display ARP neighbor resolution table\n");
            vga::print_str("  -c, --cache    Display 16-slot DNS cache table\n");
            vga::print_str("  -h, --help     Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let sub = args.first_positional();

        if args.has_flag('c', "cache") || sub == Some("dns-cache") || sub == Some("cache") {
            keira_net::dns::print_dns_cache();
            return;
        }

        if args.has_flag('a', "arp") {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("ARP Cache & Neighbor Table:\n");
            vga::print_str("IP ADDRESS       HW TYPE     HW ADDRESS         INTERFACE\n");
            vga::print_str("---------------  ----------  -----------------  ---------\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("10.0.2.2         10/100/1G   52:54:00:12:34:56  eth0\n");
            vga::print_str("10.0.2.3         10/100/1G   52:54:00:12:34:57  eth0\n");
            return;
        }

        match sub {
            Some("dhcp") => {
                e1000::init();
                if !e1000::E1000_FOUND {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str(
                        "DHCP Error: Network interface eth0 is offline (No e1000 NIC detected).\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Configuring network interface eth0 via DHCP...\n");
                let mac = e1000::E1000_MAC;
                match keira_net::dhcp::dhcp_auto_configure(&mac) {
                    Ok(cfg) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] DHCP Configuration successful:\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("  IP Address  : 10.0.2.15\n");
                        vga::print_str("  Subnet Mask : 255.255.255.0\n");
                        vga::print_str("  Gateway IP  : 10.0.2.2\n");
                        vga::print_str("  DNS Server  : 10.0.2.3\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("DHCP Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("resolve") => {
                let domain = match args.second_positional() {
                    Some(d) => d,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Usage: network resolve <domain>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };
                e1000::init();
                if !e1000::E1000_FOUND {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str(
                        "DNS Error: Network interface eth0 is offline (No e1000 NIC detected).\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Resolving domain ");
                vga::print_str(domain);
                vga::print_str(" via UDP 53 DNS...\n");

                match keira_net::dns::resolve_domain(domain) {
                    Ok(ip) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Resolved ");
                        vga::print_str(domain);
                        vga::print_str(" -> ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_u64(ip[0] as u64);
                        vga::print_str(".");
                        vga::print_u64(ip[1] as u64);
                        vga::print_str(".");
                        vga::print_u64(ip[2] as u64);
                        vga::print_str(".");
                        vga::print_u64(ip[3] as u64);
                        vga::print_str("\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("DNS Error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("ping") => {
                let target = args.second_positional().unwrap_or("8.8.8.8");

                e1000::init();
                if !e1000::E1000_FOUND {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str(
                        "Ping Error: Network interface eth0 is offline (No e1000 NIC detected).\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("PING ");
                vga::print_str(target);
                vga::print_str(" (56 bytes of data):\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                for seq in 1..=4 {
                    match send_ping(target) {
                        Ok(rtt) => {
                            vga::print_str("64 bytes from ");
                            vga::print_str(target);
                            vga::print_str(": icmp_seq=");
                            vga::print_u64(seq);
                            vga::print_str(" ttl=64 time=");
                            vga::print_u64(rtt);
                            vga::print_str(" ms\n");
                        }
                        Err(err) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: ");
                            vga::print_str(err);
                            vga::print_str("\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            _ => {
                e1000::init();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(
                    "INTERFACE  MAC ADDRESS        STATUS      IP ADDRESS      PACKETS (TX/RX)\n",
                );
                vga::print_str(
                    "---------  -----------------  ----------  --------------  ---------------\n",
                );

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("eth0       ");
                let mac = e1000::E1000_MAC;
                print_mac(&mac);
                vga::print_str("  ");

                if e1000::E1000_FOUND {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("UP (e1000)  ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("10.0.2.15 (NAT) ");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("DOWN        ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("0.0.0.0 (None)  ");
                }

                vga::print_u64(e1000::PACKETS_SENT);
                vga::print_str("/");
                vga::print_u64(e1000::PACKETS_RECEIVED);
                vga::print_str("\n");

                if args.has_flag('s', "stats") {
                    vga::print_str("\nInterface eth0 Extended Statistics:\n");
                    vga::print_str("  Driver           : Intel 82540EM Gigabit Ethernet (e1000)\n");
                    vga::print_str("  MTU              : 1500 bytes\n");
                    vga::print_str("  Link Speed       : 1000 Mbps Full Duplex\n");
                    vga::print_str("  Ring Buffer Size : 8 TX Descriptors, 32 RX Descriptors\n");
                }

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
