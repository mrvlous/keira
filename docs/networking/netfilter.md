<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# In-Kernel Stateful NAT Firewall & Packet Filtering Engine (Netfilter)

This document details the stateful packet filtering and 1:N NAT masquerading subsystem in Keira Kernel.

## 1. Subsystem Architecture
The Netfilter engine ([netfilter.rs](../../kernel/src/net/netfilter.rs)) inspects incoming and outgoing IPv4 Ethernet packets across network interfaces (`e1000`).

*   **Chains**: Evaluates rules across `INPUT`, `OUTPUT`, `FORWARD`, and `PREROUTING` hooks.
*   **Stateful Connection Tracking (`CONNTRACK`)**: Tracks TCP 3-way handshake states (`NEW`, `ESTABLISHED`, `RELATED`, `INVALID`) and UDP dynamic port mappings.
*   **Network Address Translation (NAT)**: Performs 1:1 and 1:N IP/Port masquerading for private network subnets.

---

## 2. Rule Evaluation
Packets traversing the network stack undergo sequential evaluation against chain rule tables. Actions include `ACCEPT`, `DROP`, `REJECT`, `LOG`, and `MASQUERADE`.

---

## 3. System Call & Shell Commands
*   **System Call 76 (`sys_netfilter`)**: `(cmd: u32, arg1: u64, arg2: u64) -> status`
*   **`iptables`**: Add, delete, list, or flush chain rule policies.
*   **`firewall`**: Toggle firewall state or display active connection tracking telemetry.
