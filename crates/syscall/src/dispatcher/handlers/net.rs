// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Network stack, socket, and TLS connection system call handlers.

use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::{FileDescriptor, MAX_FDS};

use crate::user_copy::{
    copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EAFNOSUPPORT, EBADF, EFAULT,
    EINVAL, EIO, EMFILE, ENFILE, ENOMEM, EPROTONOSUPPORT, ESRCH,
};

/// Syscall 24: Create an endpoint for communication.
pub fn handle_socket(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        let sock_res = keira_net::socket::create_socket(arg1, arg2, arg3);
        match sock_res {
            Ok(sock_id) => {
                let task = &mut TASKS[CURRENT_TASK_IDX];
                if let Some(t) = task {
                    let max_open_limit =
                        (t.rlimits[keira_task::rlimit::RLIMIT_NOFILE].cur as usize).min(MAX_FDS);
                    let mut free_fd = None;
                    for i in 3..max_open_limit {
                        if !t.fds[i].is_open {
                            free_fd = Some(i);
                            break;
                        }
                    }
                    if let Some(fd) = free_fd {
                        t.fds[fd] = FileDescriptor::new_socket(sock_id as u32, false);
                        fd as u64
                    } else {
                        let _ = keira_net::socket::close_socket(sock_id);
                        errno_to_ret(EMFILE)
                    }
                } else {
                    errno_to_ret(ESRCH)
                }
            }
            Err(_) => errno_to_ret(ENOMEM),
        }
    }
}

/// Syscall 25: Initiate a connection on a socket.
pub fn handle_connect(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        let fd = arg1 as usize;
        let addr_len = arg3 as usize;
        if addr_len < 8 {
            return errno_to_ret(EINVAL);
        }
        if let Err(e) = validate_user_ptr(arg2, addr_len as u64, false) {
            return errno_to_ret(e);
        }
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if fd < MAX_FDS && t.fds[fd].is_open && t.fds[fd].is_socket {
                let sock_id = t.fds[fd].socket_id as u64;
                match keira_net::socket::connect_socket(sock_id, arg2 as *const u8, arg3) {
                    Ok(()) => 0,
                    Err(_) => errno_to_ret(EINVAL),
                }
            } else {
                errno_to_ret(EBADF)
            }
        } else {
            errno_to_ret(ESRCH)
        }
    }
}

/// Syscall 33: Establish a secure TLS connection.
pub fn handle_tls_connect(arg1: u64) -> u64 {
    let host_ptr = arg1 as *const u8;
    let mut host_buf = [0u8; 64];
    let len = match unsafe { read_user_string(host_ptr, &mut host_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    if let Ok(host_str) = core::str::from_utf8(&host_buf[..len]) {
        if keira_net::tls::tls_connect(host_str).is_ok() {
            0
        } else {
            errno_to_ret(EIO)
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 76: Configure kernel firewall and packet filtering rules.
pub fn handle_netfilter(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let cmd = arg1 as u32;
    if cmd == keira_net::filter::NETFILTER_CMD_ADD_RULE && arg2 != 0 {
        let rule_ptr = arg2 as *const u8;
        let mut spec_buf = [0u8; 128];
        let len = match unsafe { read_user_string(rule_ptr, &mut spec_buf) } {
            Ok(l) => l,
            Err(e) => return errno_to_ret(e),
        };
        if let Ok(spec_str) = core::str::from_utf8(&spec_buf[..len]) {
            let mut parts = spec_str.split_whitespace();
            let chain = parts.next().unwrap_or("INPUT");
            let proto = parts.next().unwrap_or("TCP");
            let dport = parts
                .next()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            let action = parts.next().unwrap_or("ACCEPT");
            let src_ip = parts.next().unwrap_or("0.0.0.0/0");
            let dst_ip = parts.next().unwrap_or("0.0.0.0/0");

            return match keira_net::filter::add_rule(chain, proto, dport, action, src_ip, dst_ip) {
                Ok(idx) => idx as u64,
                Err(_) => errno_to_ret(EINVAL),
            };
        } else {
            return errno_to_ret(EINVAL);
        }
    }

    unsafe { keira_net::filter::sys_netfilter(cmd, arg2, arg3).unwrap_or(errno_to_ret(EINVAL)) }
}

/// Syscall 89: Transmit an ICMP ping packet to target IPv4 address and return round-trip latency.
pub fn handle_ping(arg1: u64, _arg2: u64) -> u64 {
    let target_ip_ptr = arg1 as *const u8;
    let mut ip_buf = [0u8; 64];
    let len = match unsafe { read_user_string(target_ip_ptr, &mut ip_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    if let Ok(target_str) = core::str::from_utf8(&ip_buf[..len]) {
        unsafe {
            match keira_net::icmp::send_ping(target_str) {
                Ok(rtt_ms) => rtt_ms,
                Err(_) => errno_to_ret(EIO),
            }
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 105: Create an interconnected pair of anonymous sockets.
pub fn handle_socketpair(domain: u64, socket_type: u64, protocol: u64, sv_ptr: u64) -> u64 {
    unsafe {
        if let Err(e) = validate_user_ptr(sv_ptr, 8, true) {
            return errno_to_ret(e);
        }

        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            let max_open_limit =
                (t.rlimits[keira_task::rlimit::RLIMIT_NOFILE].cur as usize).min(MAX_FDS);

            let mut free_fds = [None, None];
            let mut found = 0;
            for i in 3..max_open_limit {
                if !t.fds[i].is_open {
                    free_fds[found] = Some(i);
                    found += 1;
                    if found == 2 {
                        break;
                    }
                }
            }

            if found < 2 {
                return errno_to_ret(EMFILE);
            }

            let fd1 = free_fds[0].unwrap();
            let fd2 = free_fds[1].unwrap();

            match keira_net::socket::create_socketpair(domain, socket_type, protocol) {
                Ok((sock1_id, sock2_id)) => {
                    t.fds[fd1] = FileDescriptor::new_socket(sock1_id as u32, false);
                    t.fds[fd2] = FileDescriptor::new_socket(sock2_id as u32, false);

                    let fds: [i32; 2] = [fd1 as i32, fd2 as i32];
                    let fds_bytes = core::slice::from_raw_parts(fds.as_ptr() as *const u8, 8);
                    if copy_to_user(sv_ptr, fds_bytes).is_err() {
                        let _ = keira_net::socket::close_socket(sock1_id);
                        let _ = keira_net::socket::close_socket(sock2_id);
                        t.fds[fd1] = FileDescriptor::new();
                        t.fds[fd2] = FileDescriptor::new();
                        return errno_to_ret(EFAULT);
                    }
                    0
                }
                Err("Domain not supported") => errno_to_ret(EAFNOSUPPORT),
                Err("Socket type not supported") => errno_to_ret(EPROTONOSUPPORT),
                Err(_) => errno_to_ret(ENFILE),
            }
        } else {
            errno_to_ret(ESRCH)
        }
    }
}
