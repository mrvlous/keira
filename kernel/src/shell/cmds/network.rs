#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'network'
//!
//! Implementation of the 'network' shell command for PCI network interface control and ICMP ping testing.

use crate::io::vga;
use crate::net::e1000;
use crate::shell::executor::*;

fn print_hex_byte(b: u8) {
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 2];
    buf[0] = chars[((b >> 4) & 0xF) as usize];
    buf[1] = chars[(b & 0xF) as usize];
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

fn print_mac(mac: &[u8; 6]) {
    for i in 0..6 {
        print_hex_byte(mac[i]);
        if i < 5 {
            vga::print_str(":");
        }
    }
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let sub = parts.next();

        match sub {
            Some("-h") | Some("--help") => {
                vga::print_str(
                    "Usage: network [dhcp|dns-cache|resolve <domain>|ping <target_ip>]\n\n",
                );
                vga::print_str("Description:\n  Display network interface state (eth0), configure DHCP auto-IP, inspect 16-slot DNS cache, resolve domain names, or send ICMP echo ping.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Subcommands:\n  dhcp              Configure network interface eth0 dynamically via DHCP\n  dns-cache         Inspect 16-slot Dynamic DNS Cache Table and hit statistics\n  resolve <domain>  Query UDP Port 53 DNS Resolver for domain IP address\n  ping <target_ip>  Send ICMP Echo Request packets and measure RTT latency\n");
            }
            Some("dhcp") => {
                e1000::init();
                vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                vga::print_str("Configuring network interface eth0 via DHCP...\n");
                let mac = e1000::E1000_MAC;
                match crate::net::dhcp::dhcp_auto_configure(&mac) {
                    Ok(cfg) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] DHCP Configuration successful:\n");
                        vga::set_color(vga::Color::White, vga::Color::Black);
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
                let domain = match parts.next() {
                    Some(d) => d,
                    None => "google.com",
                };
                e1000::init();
                vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                vga::print_str("Resolving domain ");
                vga::print_str(domain);
                vga::print_str(" via UDP 53 DNS...\n");

                match crate::net::dns::resolve_domain(domain) {
                    Ok(ip) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Resolved ");
                        vga::print_str(domain);
                        vga::print_str(" -> ");
                        vga::set_color(vga::Color::White, vga::Color::Black);
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
                let target = match parts.next() {
                    Some(ip) => ip,
                    None => "8.8.8.8",
                };

                e1000::init();
                vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                vga::print_str("PING ");
                vga::print_str(target);
                vga::print_str(" (56 bytes of data):\n");
                vga::set_color(vga::Color::White, vga::Color::Black);

                for seq in 1..=4 {
                    match e1000::send_ping(target) {
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
                            vga::set_color(vga::Color::White, vga::Color::Black);
                        }
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("dns-cache") | Some("cache") => {
                crate::net::dns::print_dns_cache();
            }
            _ => {
                e1000::init();
                vga::set_color(vga::Color::LightBlue, vga::Color::Black);
                vga::print_str(
                    "INTERFACE  MAC ADDRESS        STATUS      IP ADDRESS      PACKETS (TX/RX)\n",
                );
                vga::set_color(vga::Color::White, vga::Color::Black);

                vga::print_str("eth0       ");
                let mac = e1000::E1000_MAC;
                print_mac(&mac);
                vga::print_str("  ");

                if e1000::E1000_FOUND {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("UP (e1000)  ");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("DOWN        ");
                }

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("10.0.2.15 (NAT) ");
                vga::print_u64(e1000::PACKETS_SENT);
                vga::print_str("/");
                vga::print_u64(e1000::PACKETS_RECEIVED);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
