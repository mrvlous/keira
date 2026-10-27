// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Network subsystem callback hooks enabling ProcFS real bare-metal telemetry reflection.

/// Callback provider function type for synthesizing network telemetry into a buffer.
pub type NetTelemetryProvider = fn(buf: &mut [u8]) -> usize;

/// Registered hook for live in-kernel routing table reflection (/proc/net/route).
pub static mut NET_ROUTE_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel ARP neighbor cache reflection (/proc/net/arp).
pub static mut NET_ARP_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel device metrics reflection (/proc/net/dev).
pub static mut NET_DEV_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel TCP socket table reflection (/proc/net/tcp).
pub static mut NET_TCP_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel UDP socket reflection (/proc/net/udp).
pub static mut NET_UDP_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel RAW socket reflection (/proc/net/raw).
pub static mut NET_RAW_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel Netfilter rules reflection (/proc/net/iptables).
pub static mut NET_IPTABLES_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel connection tracking metrics (/proc/net/stat/nf_conntrack).
pub static mut NET_CONNTRACK_HOOK: Option<NetTelemetryProvider> = None;

/// Registered hook for live in-kernel socket allocation summary (/proc/net/sockstat).
pub static mut NET_SOCKSTAT_HOOK: Option<NetTelemetryProvider> = None;

/// Registers live network subsystem callback hooks for dynamic bare-metal introspection.
pub fn register_net_hooks(
    route_hook: Option<NetTelemetryProvider>,
    arp_hook: Option<NetTelemetryProvider>,
    dev_hook: Option<NetTelemetryProvider>,
    tcp_hook: Option<NetTelemetryProvider>,
    udp_hook: Option<NetTelemetryProvider>,
    raw_hook: Option<NetTelemetryProvider>,
    iptables_hook: Option<NetTelemetryProvider>,
    conntrack_hook: Option<NetTelemetryProvider>,
    sockstat_hook: Option<NetTelemetryProvider>,
) {
    unsafe {
        if route_hook.is_some() {
            NET_ROUTE_HOOK = route_hook;
        }
        if arp_hook.is_some() {
            NET_ARP_HOOK = arp_hook;
        }
        if dev_hook.is_some() {
            NET_DEV_HOOK = dev_hook;
        }
        if tcp_hook.is_some() {
            NET_TCP_HOOK = tcp_hook;
        }
        if udp_hook.is_some() {
            NET_UDP_HOOK = udp_hook;
        }
        if raw_hook.is_some() {
            NET_RAW_HOOK = raw_hook;
        }
        if iptables_hook.is_some() {
            NET_IPTABLES_HOOK = iptables_hook;
        }
        if conntrack_hook.is_some() {
            NET_CONNTRACK_HOOK = conntrack_hook;
        }
        if sockstat_hook.is_some() {
            NET_SOCKSTAT_HOOK = sockstat_hook;
        }
    }
}

/// Callback provider function type for reading the active ASLR policy byte.
pub type AslrPolicyProvider = fn() -> u8;

/// Registered hook for live ASLR policy reflection (/proc/sys/kernel/randomize_va_space).
pub static mut ASLR_POLICY_HOOK: Option<AslrPolicyProvider> = None;

/// Registers live ASLR policy hook for ProcFS reflection.
pub fn register_aslr_hook(hook: Option<AslrPolicyProvider>) {
    unsafe {
        ASLR_POLICY_HOOK = hook;
    }
}
