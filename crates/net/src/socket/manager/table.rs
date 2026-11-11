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

//! In-kernel socket table manager and descriptor operations.

use crate::socket::handle::{SocketEntry, MAX_SOCKETS, SOCKET_BUF_SIZE};
use crate::tcp::state::{get_next_src_port, TcpState};

pub static mut SOCKET_TABLE: [SocketEntry; MAX_SOCKETS] = [SocketEntry::empty(); MAX_SOCKETS];
pub static mut NEXT_SOCKET_ID: u32 = 1;

/// Create a new network socket descriptor in the kernel socket table.
///
/// # Safety
/// Directly mutates static `SOCKET_TABLE` table.
pub unsafe fn create_socket(
    domain: u64,
    socket_type: u64,
    proto: u64,
) -> Result<u64, &'static str> {
    for slot in SOCKET_TABLE.iter_mut() {
        if !slot.in_use {
            let id = NEXT_SOCKET_ID;
            NEXT_SOCKET_ID = NEXT_SOCKET_ID.wrapping_add(1);
            if NEXT_SOCKET_ID == 0 {
                NEXT_SOCKET_ID = 1;
            }

            *slot = SocketEntry {
                id,
                domain: domain as u32,
                socket_type: socket_type as u32,
                protocol: proto as u32,
                state: TcpState::Closed,
                nonblocking: false,
                local_ip: [10, 0, 2, 15],
                remote_ip: [0; 4],
                local_port: 0,
                remote_port: 0,
                rx_buf: [0; SOCKET_BUF_SIZE],
                rx_len: 0,
                tx_buf: [0; SOCKET_BUF_SIZE],
                tx_len: 0,
                in_use: true,
                is_connected: false,
                peer_id: 0,
                path: [0; crate::socket::handle::UNIX_PATH_MAX],
                path_len: 0,
            };
            return Ok(id as u64);
        }
    }
    Err("Socket table exhausted")
}

/// Create an interconnected pair of anonymous UNIX domain sockets.
///
/// # Safety
/// Directly mutates static `SOCKET_TABLE` table.
pub unsafe fn create_socketpair(
    domain: u64,
    socket_type: u64,
    proto: u64,
) -> Result<(u64, u64), &'static str> {
    if domain != crate::socket::family::domains::AF_UNIX as u64 {
        return Err("Domain not supported");
    }
    if socket_type != crate::socket::family::domains::SOCK_STREAM as u64
        && socket_type != crate::socket::family::domains::SOCK_DGRAM as u64
    {
        return Err("Socket type not supported");
    }

    let mut free_slots = [None, None];
    let mut found = 0;
    for (idx, slot) in SOCKET_TABLE.iter().enumerate() {
        if !slot.in_use {
            free_slots[found] = Some(idx);
            found += 1;
            if found == 2 {
                break;
            }
        }
    }

    if found < 2 {
        return Err("Socket table exhausted");
    }

    let idx1 = free_slots[0].unwrap();
    let idx2 = free_slots[1].unwrap();

    let id1 = NEXT_SOCKET_ID;
    NEXT_SOCKET_ID = NEXT_SOCKET_ID.wrapping_add(1);
    if NEXT_SOCKET_ID == 0 {
        NEXT_SOCKET_ID = 1;
    }

    let id2 = NEXT_SOCKET_ID;
    NEXT_SOCKET_ID = NEXT_SOCKET_ID.wrapping_add(1);
    if NEXT_SOCKET_ID == 0 {
        NEXT_SOCKET_ID = 1;
    }

    let mut entry1 = SocketEntry::empty();
    entry1.id = id1;
    entry1.domain = domain as u32;
    entry1.socket_type = socket_type as u32;
    entry1.protocol = proto as u32;
    entry1.state = TcpState::Established;
    entry1.in_use = true;
    entry1.is_connected = true;
    entry1.peer_id = id2;

    let mut entry2 = SocketEntry::empty();
    entry2.id = id2;
    entry2.domain = domain as u32;
    entry2.socket_type = socket_type as u32;
    entry2.protocol = proto as u32;
    entry2.state = TcpState::Established;
    entry2.in_use = true;
    entry2.is_connected = true;
    entry2.peer_id = id1;

    SOCKET_TABLE[idx1] = entry1;
    SOCKET_TABLE[idx2] = entry2;

    Ok((id1 as u64, id2 as u64))
}

/// Bind an AF_UNIX socket to a filesystem pathname.
///
/// # Safety
/// Mutates slot in static `SOCKET_TABLE`.
pub unsafe fn bind_socket(sockfd: u64, addr_ptr: *const u8, len: u64) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if slot.domain != crate::socket::family::domains::AF_UNIX {
                return Err("Operation not supported");
            }
            if !addr_ptr.is_null() && len >= 2 {
                let path_len = (len as usize)
                    .saturating_sub(2)
                    .min(crate::socket::handle::UNIX_PATH_MAX);
                slot.path = [0; crate::socket::handle::UNIX_PATH_MAX];
                if path_len > 0 {
                    core::ptr::copy_nonoverlapping(
                        addr_ptr.add(2),
                        slot.path.as_mut_ptr(),
                        path_len,
                    );
                }
                slot.path_len = path_len;
                return Ok(());
            }
            return Err("Invalid address");
        }
    }
    Err("Invalid socket descriptor")
}

/// Connect a socket descriptor to a remote network address.
///
/// # Safety
/// Mutates target socket slot in static `SOCKET_TABLE`.
pub unsafe fn connect_socket(
    sockfd: u64,
    addr_ptr: *const u8,
    len: u64,
) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if slot.domain == crate::socket::family::domains::AF_UNIX {
                if !addr_ptr.is_null() && len >= 2 {
                    let mut target_path = [0u8; crate::socket::handle::UNIX_PATH_MAX];
                    let path_len = (len as usize)
                        .saturating_sub(2)
                        .min(crate::socket::handle::UNIX_PATH_MAX);
                    if path_len > 0 {
                        core::ptr::copy_nonoverlapping(
                            addr_ptr.add(2),
                            target_path.as_mut_ptr(),
                            path_len,
                        );
                    }
                    let mut target_id = None;
                    for other in SOCKET_TABLE.iter() {
                        if other.in_use
                            && other.id != id
                            && other.domain == crate::socket::family::domains::AF_UNIX
                            && other.path_len > 0
                            && &other.path[..other.path_len] == &target_path[..other.path_len]
                        {
                            target_id = Some(other.id);
                            break;
                        }
                    }
                    if let Some(peer_id) = target_id {
                        slot.peer_id = peer_id;
                        slot.is_connected = true;
                        slot.state = TcpState::Established;
                        for other in SOCKET_TABLE.iter_mut() {
                            if other.in_use && other.id == peer_id {
                                other.peer_id = id;
                                other.is_connected = true;
                                other.state = TcpState::Established;
                                break;
                            }
                        }
                        return Ok(());
                    } else {
                        return Err("Connection refused");
                    }
                }
                return Err("Invalid address");
            }

            if !addr_ptr.is_null() && len >= 8 {
                let port_bytes = [*(addr_ptr.add(2)), *(addr_ptr.add(3))];
                let port = u16::from_be_bytes(port_bytes);
                let ip = [
                    *(addr_ptr.add(4)),
                    *(addr_ptr.add(5)),
                    *(addr_ptr.add(6)),
                    *(addr_ptr.add(7)),
                ];
                slot.remote_ip = ip;
                slot.remote_port = port;
                slot.local_port = get_next_src_port();
                if crate::driver::loopback::is_loopback_addr(&ip) {
                    slot.local_ip = crate::driver::loopback::LOOPBACK_IP;
                } else {
                    slot.local_ip = [10, 0, 2, 15];
                }
            }
            slot.state = TcpState::Established;
            slot.is_connected = true;
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Transmit data through an established network socket.
///
/// # Safety
/// Copies memory into socket transmit buffer.
pub unsafe fn send_socket(sockfd: u64, buf: *const u8, len: usize) -> Result<usize, &'static str> {
    if buf.is_null() {
        return Err("Bad buffer pointer");
    }
    let id = sockfd as u32;

    let (is_unix, peer_id, is_connected) = {
        let mut found = None;
        for slot in SOCKET_TABLE.iter() {
            if slot.in_use && slot.id == id {
                found = Some((
                    slot.domain == crate::socket::family::domains::AF_UNIX,
                    slot.peer_id,
                    slot.is_connected,
                ));
                break;
            }
        }
        match found {
            Some(v) => v,
            None => return Err("Invalid socket descriptor"),
        }
    };

    if is_unix {
        if !is_connected || peer_id == 0 {
            return Err("Socket is not connected");
        }
        for peer in SOCKET_TABLE.iter_mut() {
            if peer.in_use && peer.id == peer_id {
                let avail = SOCKET_BUF_SIZE.saturating_sub(peer.rx_len);
                let to_write = core::cmp::min(len, avail);
                if to_write > 0 {
                    let dst = peer.rx_buf.as_mut_ptr().add(peer.rx_len);
                    core::ptr::copy_nonoverlapping(buf, dst, to_write);
                    peer.rx_len += to_write;
                    return Ok(to_write);
                } else {
                    return Err("Peer receive buffer full");
                }
            }
        }
        return Err("Broken pipe");
    }

    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if !slot.is_connected {
                return Err("Socket is not connected");
            }
            let avail = SOCKET_BUF_SIZE - slot.tx_len;
            let to_write = core::cmp::min(len, avail);
            if to_write > 0 {
                let dst = slot.tx_buf.as_mut_ptr().add(slot.tx_len);
                core::ptr::copy_nonoverlapping(buf, dst, to_write);
                slot.tx_len += to_write;

                if crate::driver::loopback::is_loopback_addr(&slot.remote_ip) {
                    let slice = core::slice::from_raw_parts(buf, to_write);
                    let _ = crate::driver::loopback::transmit_loopback_packet(slice);

                    let target_port = slot.remote_port;
                    let sender_id = slot.id;
                    let mut delivered = false;
                    for peer in SOCKET_TABLE.iter_mut() {
                        if peer.in_use && peer.id != sender_id && peer.local_port == target_port {
                            let rx_avail = SOCKET_BUF_SIZE - peer.rx_len;
                            let copy_sz = core::cmp::min(to_write, rx_avail);
                            if copy_sz > 0 {
                                let rx_dst = peer.rx_buf.as_mut_ptr().add(peer.rx_len);
                                core::ptr::copy_nonoverlapping(buf, rx_dst, copy_sz);
                                peer.rx_len += copy_sz;
                            }
                            delivered = true;
                            break;
                        }
                    }
                    if !delivered && slot.remote_port == slot.local_port {
                        let rx_avail = SOCKET_BUF_SIZE - slot.rx_len;
                        let copy_sz = core::cmp::min(to_write, rx_avail);
                        if copy_sz > 0 {
                            let rx_dst = slot.rx_buf.as_mut_ptr().add(slot.rx_len);
                            core::ptr::copy_nonoverlapping(buf, rx_dst, copy_sz);
                            slot.rx_len += copy_sz;
                        }
                    }
                } else {
                    let slice = core::slice::from_raw_parts(buf, to_write);
                    let mut res = crate::tcp::stream::client::tcp_send_and_receive(
                        slot.remote_ip,
                        slot.remote_port,
                        slice,
                    );
                    if res.is_err() && slot.remote_ip != [10, 0, 2, 2] {
                        res = crate::tcp::stream::client::tcp_send_and_receive(
                            [10, 0, 2, 2],
                            slot.remote_port,
                            slice,
                        );
                    }
                    if let Ok((resp, resp_len)) = res {
                        let rx_avail = SOCKET_BUF_SIZE.saturating_sub(slot.rx_len);
                        let copy_sz = core::cmp::min(resp_len, rx_avail);
                        if copy_sz > 0 {
                            let rx_dst = slot.rx_buf.as_mut_ptr().add(slot.rx_len);
                            core::ptr::copy_nonoverlapping(resp.as_ptr(), rx_dst, copy_sz);
                            slot.rx_len += copy_sz;
                        }
                    }
                }
            }
            return Ok(to_write);
        }
    }
    Err("Invalid socket descriptor")
}

/// Receive incoming data from a network socket buffer.
///
/// # Safety
/// Copies received bytes into caller-provided buffer.
pub unsafe fn recv_socket(sockfd: u64, buf: *mut u8, len: usize) -> Result<usize, &'static str> {
    if buf.is_null() {
        return Err("Bad buffer pointer");
    }
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if slot.rx_len == 0 {
                if slot.domain == crate::socket::family::domains::AF_UNIX
                    && (!slot.is_connected || slot.peer_id == 0)
                {
                    return Ok(0);
                }
                if slot.nonblocking {
                    return Err("Resource temporarily unavailable");
                }
                return Ok(0);
            }
            let to_read = core::cmp::min(len, slot.rx_len);
            core::ptr::copy_nonoverlapping(slot.rx_buf.as_ptr(), buf, to_read);
            if to_read < slot.rx_len {
                let remaining = slot.rx_len - to_read;
                core::ptr::copy(
                    slot.rx_buf.as_ptr().add(to_read),
                    slot.rx_buf.as_mut_ptr(),
                    remaining,
                );
            }
            slot.rx_len -= to_read;
            return Ok(to_read);
        }
    }
    Err("Invalid socket descriptor")
}

/// Close and release a socket descriptor from the active table.
///
/// # Safety
/// Mutates slot in static `SOCKET_TABLE`.
pub unsafe fn close_socket(sockfd: u64) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            let peer_id = slot.peer_id;
            *slot = SocketEntry::empty();
            if peer_id != 0 {
                for peer in SOCKET_TABLE.iter_mut() {
                    if peer.in_use && peer.id == peer_id {
                        peer.peer_id = 0;
                        peer.is_connected = false;
                        peer.state = TcpState::Closed;
                    }
                }
            }
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Check whether the specified socket descriptor has incoming data available or has connected.
///
/// # Safety
/// Reads static `SOCKET_TABLE`.
pub unsafe fn socket_is_readable(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.rx_len > 0 || slot.is_connected;
        }
    }
    false
}

/// Check whether the specified socket descriptor can accept outgoing transmission bytes.
///
/// # Safety
/// Reads static `SOCKET_TABLE`.
pub unsafe fn socket_is_writable(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.is_connected && slot.tx_len < SOCKET_BUF_SIZE;
        }
    }
    false
}

/// Configure non-blocking I/O mode for the specified socket descriptor.
///
/// # Safety
/// Mutates slot in static `SOCKET_TABLE`.
pub unsafe fn set_socket_nonblocking(sockfd: u64, nonblocking: bool) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            slot.nonblocking = nonblocking;
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Query whether the specified socket descriptor is configured in non-blocking mode.
///
/// # Safety
/// Reads static `SOCKET_TABLE`.
pub unsafe fn is_socket_nonblocking(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.nonblocking;
        }
    }
    false
}

/// Enqueue simulated or driver-received network payload into a socket receive buffer.
///
/// # Safety
/// Writes bytes into socket buffer in static `SOCKET_TABLE`.
pub unsafe fn queue_rx_data(sockfd: u64, data: &[u8]) -> Result<usize, &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            let avail = SOCKET_BUF_SIZE - slot.rx_len;
            let to_copy = core::cmp::min(data.len(), avail);
            if to_copy > 0 {
                let dst = slot.rx_buf.as_mut_ptr().add(slot.rx_len);
                core::ptr::copy_nonoverlapping(data.as_ptr(), dst, to_copy);
                slot.rx_len += to_copy;
            }
            return Ok(to_copy);
        }
    }
    Err("Invalid socket descriptor")
}

/// Reset socket table state (used for testing and teardown).
///
/// # Safety
/// Clears entire static `SOCKET_TABLE`.
pub unsafe fn socket_table_reset() {
    SOCKET_TABLE = [SocketEntry::empty(); MAX_SOCKETS];
    NEXT_SOCKET_ID = 1;
}

/// Formats live in-kernel TCP sockets into canonical `/proc/net/tcp` format.
pub fn format_proc_tcp(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode"
    );

    unsafe {
        let mut sl = 0;
        for entry in SOCKET_TABLE.iter() {
            if entry.in_use && entry.socket_type == 1 {
                let loc_ip = u32::from_ne_bytes(entry.local_ip);
                let rem_ip = u32::from_ne_bytes(entry.remote_ip);
                let st_num: u8 = match entry.state {
                    TcpState::Established => 0x01,
                    TcpState::SynSent => 0x02,
                    TcpState::FinWait => 0x04,
                    TcpState::Closed => 0x07,
                };
                let _ = writeln!(
                    writer,
                    "{:>4}: {:08X}:{:04X} {:08X}:{:04X} {:02X} {:08X}:{:08X} 00:00000000 00000000     0        0 {}",
                    sl, loc_ip, entry.local_port, rem_ip, entry.remote_port, st_num, entry.tx_len, entry.rx_len, entry.id
                );
                sl += 1;
            }
        }
    }

    writer.o
}

/// Formats live in-kernel UDP sockets into canonical `/proc/net/udp` format.
pub fn format_proc_udp(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops"
    );

    unsafe {
        let mut sl = 0;
        for entry in SOCKET_TABLE.iter() {
            if entry.in_use && entry.socket_type == 2 {
                let loc_ip = u32::from_ne_bytes(entry.local_ip);
                let rem_ip = u32::from_ne_bytes(entry.remote_ip);
                let _ = writeln!(
                    writer,
                    "{:>4}: {:08X}:{:04X} {:08X}:{:04X} 07 {:08X}:{:08X} 00:00000000 00000000     0        0 {} 1 0000000000000000 0",
                    sl, loc_ip, entry.local_port, rem_ip, entry.remote_port, entry.tx_len, entry.rx_len, entry.id
                );
                sl += 1;
            }
        }
    }

    writer.o
}

/// Formats live in-kernel RAW sockets into canonical `/proc/net/raw` format.
pub fn format_proc_raw(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops"
    );

    unsafe {
        let mut sl = 0;
        for entry in SOCKET_TABLE.iter() {
            if entry.in_use && entry.socket_type == 3 {
                let loc_ip = u32::from_ne_bytes(entry.local_ip);
                let rem_ip = u32::from_ne_bytes(entry.remote_ip);
                let _ = writeln!(
                    writer,
                    "{:>4}: {:08X}:{:04X} {:08X}:{:04X} 07 {:08X}:{:08X} 00:00000000 00000000     0        0 {} 1 0000000000000000 0",
                    sl, loc_ip, entry.local_port, rem_ip, entry.remote_port, entry.tx_len, entry.rx_len, entry.id
                );
                sl += 1;
            }
        }
    }

    writer.o
}

/// Formats socket allocation statistics into canonical `/proc/net/sockstat` format.
pub fn format_proc_sockstat(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let mut total_used = 0;
    let mut tcp_inuse = 0;
    let mut udp_inuse = 0;
    let mut raw_inuse = 0;

    unsafe {
        for entry in SOCKET_TABLE.iter() {
            if entry.in_use {
                total_used += 1;
                match entry.socket_type {
                    1 => tcp_inuse += 1,
                    2 => udp_inuse += 1,
                    3 => raw_inuse += 1,
                    _ => {}
                }
            }
        }
    }

    let _ = writeln!(writer, "sockets: used {}", total_used);
    let _ = writeln!(
        writer,
        "TCP: inuse {} orphan 0 tw 0 alloc {} mem 1",
        tcp_inuse, tcp_inuse
    );
    let _ = writeln!(writer, "UDP: inuse {} mem 0", udp_inuse);
    let _ = writeln!(writer, "RAW: inuse {}", raw_inuse);
    let _ = writeln!(writer, "FRAG: inuse 0 memory 0");

    writer.o
}

/// Formats live in-kernel UNIX domain sockets into canonical `/proc/net/unix` format.
pub fn format_proc_unix(buf: &mut [u8]) -> usize {
    use core::fmt::Write;
    struct SliceWriter<'a> {
        b: &'a mut [u8],
        o: usize,
    }
    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.b.len().saturating_sub(self.o);
            let n = rem.min(bytes.len());
            self.b[self.o..self.o + n].copy_from_slice(&bytes[..n]);
            self.o += n;
            Ok(())
        }
    }

    let mut writer = SliceWriter { b: buf, o: 0 };
    let _ = writeln!(
        writer,
        "Num       RefCount Protocol Flags    Type St Inode Path"
    );

    unsafe {
        for slot in SOCKET_TABLE.iter() {
            if slot.in_use && slot.domain == crate::socket::family::domains::AF_UNIX {
                let st_num: u8 = if slot.is_connected { 3 } else { 1 };
                let type_num = slot.socket_type;
                let path_str = if slot.path_len > 0 {
                    core::str::from_utf8(&slot.path[..slot.path_len]).unwrap_or("[invalid]")
                } else if slot.peer_id != 0 {
                    "[socketpair]"
                } else {
                    "[unnamed]"
                };

                let _ = writeln!(
                    writer,
                    "{:08x}: 00000002 00000000 00010000 {:04x} {:02x}     0 {}",
                    slot.id, type_num, st_num, path_str
                );
            }
        }
    }

    writer.o
}

/// Iterate active UNIX domain sockets for diagnostic inspection.
///
/// # Safety
/// Reads static `SOCKET_TABLE`.
pub unsafe fn for_each_unix_socket<F: FnMut(&SocketEntry)>(mut f: F) {
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.domain == crate::socket::family::domains::AF_UNIX {
            f(slot);
        }
    }
}
