// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel IPv4 routing table (Forwarding Information Base - FIB) and longest-prefix match.

/// Route flags matching standard POSIX / Linux route table semantics.
pub const RTF_UP: u16 = 0x0001;
pub const RTF_GATEWAY: u16 = 0x0002;
pub const RTF_HOST: u16 = 0x0004;

pub const MAX_ROUTES: usize = 8;

/// Single IPv4 routing table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteEntry {
    pub dest: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway: [u8; 4],
    pub iface: &'static str,
    pub flags: u16,
    pub metric: u32,
    pub valid: bool,
}

impl RouteEntry {
    /// Constructs an uninitialized, invalid route entry.
    pub const fn empty() -> Self {
        Self {
            dest: [0; 4],
            netmask: [0; 4],
            gateway: [0; 4],
            iface: "",
            flags: 0,
            metric: 0,
            valid: false,
        }
    }

    /// Constructs a valid route entry.
    pub const fn new(
        dest: [u8; 4],
        netmask: [u8; 4],
        gateway: [u8; 4],
        iface: &'static str,
        flags: u16,
        metric: u32,
    ) -> Self {
        Self {
            dest,
            netmask,
            gateway,
            iface,
            flags,
            metric,
            valid: true,
        }
    }

    /// Calculates the CIDR prefix length (number of set bits in netmask).
    pub fn prefix_len(&self) -> u32 {
        let mut bits = 0;
        for &byte in &self.netmask {
            bits += byte.count_ones();
        }
        bits
    }

    /// Checks if a given destination IPv4 address matches this route.
    pub fn matches(&self, target: &[u8; 4]) -> bool {
        if !self.valid || (self.flags & RTF_UP) == 0 {
            return false;
        }
        for i in 0..4 {
            if (target[i] & self.netmask[i]) != self.dest[i] {
                return false;
            }
        }
        true
    }
}

/// In-kernel static Forwarding Information Base (FIB) routing table.
pub static mut ROUTING_TABLE: [RouteEntry; MAX_ROUTES] = [
    // 1. Loopback route: 127.0.0.0/8 -> lo
    RouteEntry::new(
        [127, 0, 0, 0],
        [255, 0, 0, 0],
        [0, 0, 0, 0],
        "lo",
        RTF_UP,
        0,
    ),
    // 2. Local subnet route: 10.0.2.0/24 -> eth0
    RouteEntry::new(
        [10, 0, 2, 0],
        [255, 255, 255, 0],
        [0, 0, 0, 0],
        "eth0",
        RTF_UP,
        100,
    ),
    // 3. Default gateway route: 0.0.0.0/0 -> 10.0.2.2 via eth0
    RouteEntry::new(
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [10, 0, 2, 2],
        "eth0",
        RTF_UP | RTF_GATEWAY,
        100,
    ),
    RouteEntry::empty(),
    RouteEntry::empty(),
    RouteEntry::empty(),
    RouteEntry::empty(),
    RouteEntry::empty(),
];

/// Performs longest-prefix match routing lookup for a destination IPv4 address.
///
/// # Safety
/// Reads static in-kernel `ROUTING_TABLE`.
pub unsafe fn lookup_route(target: &[u8; 4]) -> Option<RouteEntry> {
    let mut best_match: Option<RouteEntry> = None;
    let mut best_prefix = 0;
    let mut best_metric = u32::MAX;

    let table_ptr = &raw const ROUTING_TABLE;
    for i in 0..MAX_ROUTES {
        let entry = *((*table_ptr).as_ptr().add(i));
        if entry.matches(target) {
            let prefix = entry.prefix_len();
            if prefix > best_prefix
                || (prefix == best_prefix && entry.metric < best_metric)
                || best_match.is_none()
            {
                best_match = Some(entry);
                best_prefix = prefix;
                best_metric = entry.metric;
            }
        }
    }

    best_match
}

/// Adds a new route entry to the in-kernel routing table.
///
/// # Safety
/// Mutates static in-kernel `ROUTING_TABLE`.
pub unsafe fn add_route(
    dest: [u8; 4],
    netmask: [u8; 4],
    gateway: [u8; 4],
    iface: &'static str,
    flags: u16,
    metric: u32,
) -> Result<(), &'static str> {
    let table_ptr = &raw mut ROUTING_TABLE;

    // Check if duplicate exists and update it
    for i in 0..MAX_ROUTES {
        let entry = &mut *((*table_ptr).as_mut_ptr().add(i));
        if entry.valid && entry.dest == dest && entry.netmask == netmask {
            *entry = RouteEntry::new(dest, netmask, gateway, iface, flags, metric);
            return Ok(());
        }
    }

    // Allocate first empty slot
    for i in 0..MAX_ROUTES {
        let entry = &mut *((*table_ptr).as_mut_ptr().add(i));
        if !entry.valid {
            *entry = RouteEntry::new(dest, netmask, gateway, iface, flags, metric);
            return Ok(());
        }
    }

    Err("Routing table full")
}

/// Deletes a route entry matching destination and netmask.
///
/// # Safety
/// Mutates static in-kernel `ROUTING_TABLE`.
pub unsafe fn del_route(dest: &[u8; 4], netmask: &[u8; 4]) -> Result<(), &'static str> {
    let table_ptr = &raw mut ROUTING_TABLE;
    for i in 0..MAX_ROUTES {
        let entry = &mut *((*table_ptr).as_mut_ptr().add(i));
        if entry.valid && entry.dest == *dest && entry.netmask == *netmask {
            *entry = RouteEntry::empty();
            return Ok(());
        }
    }
    Err("Route not found")
}

/// Retrieves a snapshot copy of the active in-kernel routing table.
///
/// # Safety
/// Reads static in-kernel `ROUTING_TABLE`.
pub unsafe fn get_routes() -> [RouteEntry; MAX_ROUTES] {
    let table_ptr = &raw const ROUTING_TABLE;
    *table_ptr
}

/// Formats live in-kernel routing table (FIB) in canonical /proc/net/route format.
pub fn format_proc_route(buf: &mut [u8]) -> usize {
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
        "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT"
    );

    unsafe {
        let table_ptr = &raw const ROUTING_TABLE;
        for i in 0..MAX_ROUTES {
            let route = &*(*table_ptr).as_ptr().add(i);
            if !route.valid {
                continue;
            }
            let dest_u32 = u32::from_ne_bytes(route.dest);
            let gw_u32 = u32::from_ne_bytes(route.gateway);
            let mask_u32 = u32::from_ne_bytes(route.netmask);
            let _ = writeln!(
                writer,
                "{}\t{:08X}\t{:08X}\t{:04X}\t0\t0\t{}\t{:08X}\t1500\t0\t0",
                route.iface, dest_u32, gw_u32, route.flags, route.metric, mask_u32
            );
        }
    }

    writer.o
}
