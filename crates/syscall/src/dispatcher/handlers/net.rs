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
    copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EBADF, EINVAL, EIO, EMFILE,
    ENOMEM, ESRCH,
};

/// Syscall 17: Fetch remote URL content via synchronous HTTP GET request.
pub fn handle_http_get(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let url_ptr = arg1 as *const u8;
    let out_buf_ptr = arg2;
    let max_len = arg3;

    let mut url_buf = [0u8; 128];
    let len = match unsafe { read_user_string(url_ptr, &mut url_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };

    if let Ok(url_str) = core::str::from_utf8(&url_buf[..len]) {
        if let Ok((resp_buf, resp_len)) = unsafe { keira_net::tcp::stream::fetch_http(url_str) } {
            let to_copy = (max_len as usize).min(resp_len);
            if unsafe { copy_to_user(out_buf_ptr, &resp_buf[..to_copy]) }.is_ok() {
                return to_copy as u64;
            }
        }
    }
    errno_to_ret(EIO)
}

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
