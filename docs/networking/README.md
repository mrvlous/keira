<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Networking & Packet Filtering Subsystems

Welcome to the Networking documentation section for Keira Kernel.

## Documents

* [Intel e1000 Network Driver & Socket API](network.md): PCI enumeration, MAC address parsing, TCP state engine, DHCP client, Dynamic ARP cache, POSIX Sockets, and Native TLS 1.3 Engine (`https`).
* [In-Kernel DNS Resolver & Cache Table](dns_resolver.md): 16-slot dynamic LRU DNS cache table and UDP 53 RFC 1035 packet resolution.
* [In-Kernel Stateful NAT Firewall Engine](netfilter.md): Stateful IPv4 packet filtering, IPTables rules, and 1:N NAT masquerading (`sys_netfilter`).
* [Zero-Copy BPF Packet Filter Engine](bpf.md): In-kernel BPF bytecode interpreter for raw socket packet filtering.
* [eBPF JIT Compiler Engine](bpf_jit.md): Native x86_64 JIT bytecode translation (`sys_bpf_jit`).
* [io_uring Async Network Socket Polling](io_uring_net.md): Zero-copy async network socket polling (`sys_io_uring_net`).
* [POSIX PTP Hardware Clock Subsystem](ptp.md): IEEE 1588 nanosecond-precision hardware clock (`sys_ptp_clock`).
