// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and manage POSIX Message Queue descriptors & message backlog (Syscall 58).

use keira_io::vga;
use keira_ipc::mqueue::{
    get_mqueue_stats, get_mqueue_table, mq_open, mq_receive, mq_send, mq_unlink, mq_unlink_by_id,
    MAX_MQUEUES, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();
    if subcmd == Some("-h") || subcmd == Some("--help") || subcmd.is_none() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: mqueue <subcommand> [args]\n\n");
            vga::print_str(
                "Description:\n  Inspect and manage in-kernel POSIX Message Queue descriptors (Syscall 58).\n\n",
            );
            vga::print_str("Subcommands:\n");
            vga::print_str("  status                  Display message queue subsystem telemetry\n");
            vga::print_str(
                "  list                    List active message queues and message counts\n",
            );
            vga::print_str("  create <name>           Create a new in-kernel message queue\n");
            vga::print_str(
                "  send <queue> <msg>      Enqueue payload into queue (default priority 10)\n",
            );
            vga::print_str(
                "  recv <queue>            Dequeue highest priority message from queue\n",
            );
            vga::print_str("  unlink <queue>          Unlink and destroy message queue\n");
            vga::print_str("  -h, --help              Show this help message and exit\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    unsafe {
        match subcmd.unwrap() {
            "status" => {
                let (active_queues, total_msgs) = get_mqueue_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("POSIX Message Queue Subsystem Status:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Subsystem Engine : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (In-Kernel Priority Queue Engine)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Active Queues    : ");
                vga::print_u64(active_queues as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_MQUEUES as u64);
                vga::print_str(" allocated\n");
                vga::print_str("  Queued Messages  : ");
                vga::print_u64(total_msgs as u64);
                vga::print_str(" messages total\n");
                vga::print_str("  Queue Capacity   : ");
                vga::print_u64(MQUEUE_MAX_MSGS as u64);
                vga::print_str(" msgs per queue\n");
                vga::print_str("  Max Message Size : ");
                vga::print_u64(MQUEUE_MSG_SIZE as u64);
                vga::print_str(" bytes\n");
                vga::print_str("  Syscall Vector   : Syscall 58 (mq_open)\n");
            }
            "list" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("ID   NAME             MSGS   MAX_MSGS  MSG_SIZE\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let table = get_mqueue_table();
                let mut count = 0;
                for q in table.iter() {
                    if q.in_use {
                        vga::print_u64(q.id as u64);
                        vga::print_str("    ");
                        vga::print_str(q.name_as_str());
                        let len = q.name_len;
                        if len < 17 {
                            let pad = 17 - len;
                            for _ in 0..pad {
                                vga::print_str(" ");
                            }
                        } else {
                            vga::print_str(" ");
                        }
                        vga::print_u64(q.cur_msgs as u64);
                        vga::print_str("      ");
                        vga::print_u64(q.max_msg as u64);
                        vga::print_str("         ");
                        vga::print_u64(q.msg_size as u64);
                        vga::print_str(" B\n");
                        count += 1;
                    }
                }
                if count == 0 {
                    vga::print_str("  (no active POSIX message queues)\n");
                }
            }
            "create" => {
                if let Some(name) = parts.next() {
                    match mq_open(name, 0, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE) {
                        Ok(id) => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("[OK] ");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("Created POSIX Message Queue '");
                            vga::print_str(name);
                            vga::print_str("' (MQID #");
                            vga::print_u64(id as u64);
                            vga::print_str(")\n");
                        }
                        Err(e) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: Failed to create queue '");
                            vga::print_str(name);
                            vga::print_str("' (");
                            vga::print_str(e);
                            vga::print_str(")\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Queue name required. Usage: mqueue create <name>\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
            "send" => {
                let name = match parts.next() {
                    Some(n) => n,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Queue name required. Usage: mqueue send <queue> <msg>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let mut msg_buf = [0u8; MQUEUE_MSG_SIZE];
                let mut offset = 0;
                let mut first = true;
                for part in parts {
                    if !first && offset < MQUEUE_MSG_SIZE {
                        msg_buf[offset] = b' ';
                        offset += 1;
                    }
                    first = false;
                    let b = part.as_bytes();
                    let copy_len = b.len().min(MQUEUE_MSG_SIZE - offset);
                    msg_buf[offset..offset + copy_len].copy_from_slice(&b[..copy_len]);
                    offset += copy_len;
                }

                if offset == 0 {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str(
                        "Error: Message payload required. Usage: mqueue send <queue> <msg>\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                match mq_send(name, &msg_buf[..offset], 10) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Enqueued ");
                        vga::print_u64(offset as u64);
                        vga::print_str(" bytes into '");
                        vga::print_str(name);
                        vga::print_str("' (Priority: 10)\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Failed to enqueue into '");
                        vga::print_str(name);
                        vga::print_str("' (");
                        vga::print_str(e);
                        vga::print_str(")\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "recv" => {
                let name = match parts.next() {
                    Some(n) => n,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Queue name required. Usage: mqueue recv <queue>\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let mut buf = [0u8; MQUEUE_MSG_SIZE];
                match mq_receive(name, &mut buf) {
                    Ok((len, prio)) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Dequeued (Priority ");
                        vga::print_u64(prio as u64);
                        vga::print_str(", ");
                        vga::print_u64(len as u64);
                        vga::print_str(" bytes): \"");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                            vga::print_str(s);
                        } else {
                            vga::print_str("<binary>");
                        }
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("\"\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Failed to dequeue from '");
                        vga::print_str(name);
                        vga::print_str("' (");
                        vga::print_str(e);
                        vga::print_str(")\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            "unlink" => {
                let target = match parts.next() {
                    Some(t) => t,
                    None => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Queue target required. Usage: mqueue unlink <queue>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                let res = if let Ok(id) = parse_u32(target) {
                    mq_unlink_by_id(id)
                } else {
                    mq_unlink(target)
                };

                match res {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("Unlinked POSIX Message Queue '");
                        vga::print_str(target);
                        vga::print_str("'\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error: Failed to unlink '");
                        vga::print_str(target);
                        vga::print_str("' (");
                        vga::print_str(e);
                        vga::print_str(")\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
            unknown => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error: Unrecognized subcommand '");
                vga::print_str(unknown);
                vga::print_str("'. Use 'mqueue --help' for available subcommands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

fn parse_u32(s: &str) -> Result<u32, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u32 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u32).ok_or(())?;
    }
    Ok(val)
}
