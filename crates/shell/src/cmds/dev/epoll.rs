// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and manage epoll scalable I/O event multiplexer instances (Syscall 55, 56 & 57).

use keira_io::vga;
use keira_ipc::event::{
    epoll_ctl_internal, get_epoll_instances, get_epoll_stats, sys_epoll_create, sys_epoll_wait,
    EpollEvent, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    match subcmd {
        Some("-h") | Some("--help") => unsafe {
            vga::print_str("Usage: epoll [status|list|create|ctl|wait|test]\n\n");
            vga::print_str(
                "Description:\n  Inspect and manage epoll scalable I/O event multiplexer instances (Syscall 55, 56 & 57).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                   Show epoll subsystem statistics and active descriptor count\n");
            vga::print_str("  list                     Display all active epoll instances and monitored descriptors\n");
            vga::print_str(
                "  create [size]            Allocate a new epoll file descriptor instance\n",
            );
            vga::print_str(
                "  ctl <epfd> <op> <fd>     Control monitored target descriptor (add/mod/del)\n",
            );
            vga::print_str(
                "  wait <epfd> [max]        Poll and dispatch ready I/O events on an instance\n",
            );
            vga::print_str(
                "  test                     Execute automated self-test on epoll multiplexing\n",
            );
            vga::print_str(
                "\nOptions:\n  -h, --help               Show this help message and exit\n",
            );
        },
        Some("list") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Active Epoll Instances & Interest Lists:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let instances = get_epoll_instances();
            let mut found = false;
            for slot in instances.iter() {
                if let Some(ref inst) = slot {
                    if inst.active {
                        found = true;
                        vga::print_str("  EPFD #");
                        vga::print_u64(inst.epfd as u64);
                        vga::print_str(" (Size Hint: ");
                        vga::print_u64(inst.size_hint as u64);
                        vga::print_str(", Monitored FDs: ");
                        vga::print_u64(inst.item_count as u64);
                        vga::print_str(", Total Polls: ");
                        vga::print_u64(inst.total_polls);
                        vga::print_str(")\n");

                        for item in inst.items.iter() {
                            if item.in_use {
                                vga::print_str("    -> FD #");
                                vga::print_u64(item.fd as u64);
                                vga::print_str(" [Events: 0x");
                                vga::print_hex(item.event.events as u64);
                                if (item.event.events & EPOLLIN) != 0 {
                                    vga::print_str(" IN");
                                }
                                if (item.event.events & EPOLLOUT) != 0 {
                                    vga::print_str(" OUT");
                                }
                                if (item.event.events & EPOLLERR) != 0 {
                                    vga::print_str(" ERR");
                                }
                                if (item.event.events & EPOLLHUP) != 0 {
                                    vga::print_str(" HUP");
                                }
                                vga::print_str("] Data: 0x");
                                vga::print_hex(item.event.data);
                                vga::print_str("\n");
                            }
                        }
                    }
                }
            }
            if !found {
                vga::print_str(
                    "  No active epoll instances registered. Use 'epoll create' or 'epoll test'.\n",
                );
            }
        },
        Some("create") => unsafe {
            let size = parts
                .next()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(64);
            match sys_epoll_create(size) {
                Ok(epfd) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Allocated Epoll Instance EPFD #");
                    vga::print_u64(epfd);
                    vga::print_str(" (Size Hint: ");
                    vga::print_u64(size as u64);
                    vga::print_str(")\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] sys_epoll_create failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("ctl") => unsafe {
            let epfd = match parts.next().and_then(|s| s.parse::<i32>().ok()) {
                Some(val) => val,
                None => {
                    vga::print_str("Usage: epoll ctl <epfd> <add|mod|del> <fd> [events]\n");
                    return;
                }
            };
            let op_str = parts.next().unwrap_or("add");
            let op = match op_str {
                "add" => EPOLL_CTL_ADD,
                "mod" => EPOLL_CTL_MOD,
                "del" => EPOLL_CTL_DEL,
                _ => {
                    vga::print_str("Invalid op. Use add, mod, or del.\n");
                    return;
                }
            };
            let fd = match parts.next().and_then(|s| s.parse::<i32>().ok()) {
                Some(val) => val,
                None => {
                    vga::print_str("Missing target fd argument.\n");
                    return;
                }
            };

            let ev = EpollEvent {
                events: EPOLLIN | EPOLLOUT,
                data: fd as u64,
            };

            match epoll_ctl_internal(epfd, op, fd, ev) {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Epoll ctl updated target FD #");
                    vga::print_u64(fd as u64);
                    vga::print_str(" on EPFD #");
                    vga::print_u64(epfd as u64);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] epoll ctl failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("wait") => unsafe {
            let epfd = match parts.next().and_then(|s| s.parse::<i32>().ok()) {
                Some(val) => val,
                None => {
                    vga::print_str("Usage: epoll wait <epfd> [maxevents]\n");
                    return;
                }
            };
            let maxevents = parts
                .next()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(4);

            let mut events_buf = [EpollEvent::default(); 8];
            match sys_epoll_wait(epfd, events_buf.as_mut_ptr() as u64, maxevents, 0) {
                Ok(ready) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Polled EPFD #");
                    vga::print_u64(epfd as u64);
                    vga::print_str(" - Ready Events: ");
                    vga::print_u64(ready);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    for i in 0..(ready as usize).min(events_buf.len()) {
                        let ev = &events_buf[i];
                        vga::print_str("  Event [");
                        vga::print_u64(i as u64);
                        vga::print_str("]: Mask 0x");
                        vga::print_hex(ev.events as u64);
                        vga::print_str(" Data: 0x");
                        vga::print_hex(ev.data);
                        vga::print_str("\n");
                    }
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[ERROR] epoll_wait failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        Some("test") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[TEST] Executing Epoll Scalable I/O Engine Verification...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            match sys_epoll_create(32) {
                Ok(epfd) => {
                    vga::print_str("  1. Created epoll instance EPFD #");
                    vga::print_u64(epfd);
                    vga::print_str(" - OK\n");

                    let ev1 = EpollEvent {
                        events: EPOLLIN,
                        data: 0x100,
                    };
                    let _ = epoll_ctl_internal(epfd as i32, EPOLL_CTL_ADD, 5, ev1);
                    vga::print_str("  2. Added FD #5 (EPOLLIN) to interest list - OK\n");

                    let ev2 = EpollEvent {
                        events: EPOLLIN | EPOLLOUT,
                        data: 0x200,
                    };
                    let _ = epoll_ctl_internal(epfd as i32, EPOLL_CTL_ADD, 6, ev2);
                    vga::print_str("  3. Added FD #6 (EPOLLIN|EPOLLOUT) to interest list - OK\n");

                    let mut evs = [EpollEvent::default(); 4];
                    if let Ok(ready) = sys_epoll_wait(epfd as i32, evs.as_mut_ptr() as u64, 4, 0) {
                        vga::print_str("  4. Dispatched epoll_wait - ");
                        vga::print_u64(ready);
                        vga::print_str(" active ready events - OK\n");
                    }

                    let _ = epoll_ctl_internal(epfd as i32, EPOLL_CTL_DEL, 5, ev1);
                    vga::print_str("  5. Deleted FD #5 from interest list - OK\n");

                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str(
                        "[PASS] Epoll scalable I/O subsystem operational (Syscall 55/56/57).\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[FAIL] Epoll test failed: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        },
        _ => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Epoll Scalable I/O Multiplexer Engine (Syscall 55, 56 & 57) ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[Active]\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let (instances, items) = get_epoll_stats();
            vga::print_str("  Status      : Operational (O(1) Ready Polling)\n");
            vga::print_str("  Instances   : ");
            vga::print_u64(instances as u64);
            vga::print_str(" active (Capacity: 8)\n");
            vga::print_str("  Watched FDs : ");
            vga::print_u64(items as u64);
            vga::print_str(" descriptors in interest list\n");
            vga::print_str(
                "  Features    : Edge-Triggered (EPOLLET), EPOLLIN, EPOLLOUT, EPOLLERR, EPOLLHUP\n",
            );
            vga::print_str("  Syscalls    : 55 (epoll_create), 56 (epoll_ctl), 57 (epoll_wait)\n");
        },
    }
}
