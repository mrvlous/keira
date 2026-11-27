// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

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

/// Registered hook for live in-kernel UNIX domain socket table reflection (/proc/net/unix).
pub static mut NET_UNIX_HOOK: Option<NetTelemetryProvider> = None;

/// Registers live UNIX domain socket introspection hook.
pub fn register_unix_net_hook(unix_hook: Option<NetTelemetryProvider>) {
    unsafe {
        NET_UNIX_HOOK = unix_hook;
    }
}

/// Bundle of optional network telemetry callbacks for ProcFS registration.
#[derive(Default, Clone, Copy)]
pub struct NetProcHooks {
    pub route: Option<NetTelemetryProvider>,
    pub arp: Option<NetTelemetryProvider>,
    pub dev: Option<NetTelemetryProvider>,
    pub tcp: Option<NetTelemetryProvider>,
    pub udp: Option<NetTelemetryProvider>,
    pub raw: Option<NetTelemetryProvider>,
    pub iptables: Option<NetTelemetryProvider>,
    pub conntrack: Option<NetTelemetryProvider>,
    pub sockstat: Option<NetTelemetryProvider>,
}

/// Registers live network subsystem callback hooks for dynamic bare-metal introspection.
pub fn register_net_hooks(hooks: NetProcHooks) {
    unsafe {
        if hooks.route.is_some() {
            NET_ROUTE_HOOK = hooks.route;
        }
        if hooks.arp.is_some() {
            NET_ARP_HOOK = hooks.arp;
        }
        if hooks.dev.is_some() {
            NET_DEV_HOOK = hooks.dev;
        }
        if hooks.tcp.is_some() {
            NET_TCP_HOOK = hooks.tcp;
        }
        if hooks.udp.is_some() {
            NET_UDP_HOOK = hooks.udp;
        }
        if hooks.raw.is_some() {
            NET_RAW_HOOK = hooks.raw;
        }
        if hooks.iptables.is_some() {
            NET_IPTABLES_HOOK = hooks.iptables;
        }
        if hooks.conntrack.is_some() {
            NET_CONNTRACK_HOOK = hooks.conntrack;
        }
        if hooks.sockstat.is_some() {
            NET_SOCKSTAT_HOOK = hooks.sockstat;
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

/// Callback provider function type for synthesizing wait queue telemetry into a buffer.
pub type WaitQueueTelemetryProvider = fn(buf: &mut [u8]) -> usize;

/// Registered hook for live in-kernel wait queue reflection (/proc/waitqueue).
pub static mut WAITQUEUE_HOOK: Option<WaitQueueTelemetryProvider> = None;

/// Registers live wait queue introspection hook.
pub fn register_waitqueue_hook(hook: WaitQueueTelemetryProvider) {
    unsafe {
        WAITQUEUE_HOOK = Some(hook);
    }
}

/// Callback provider function type for synthesizing workqueue telemetry into a buffer.
pub type WorkQueueTelemetryProvider = fn(buf: &mut [u8]) -> usize;

/// Registered hook for live in-kernel workqueue reflection (/proc/workqueues).
pub static mut WORKQUEUES_HOOK: Option<WorkQueueTelemetryProvider> = None;

/// Registers live workqueue introspection hook.
pub fn register_workqueues_hook(hook: WorkQueueTelemetryProvider) {
    unsafe {
        WORKQUEUES_HOOK = Some(hook);
    }
}
