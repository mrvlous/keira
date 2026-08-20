<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Netfilter Firewall & in-Kernel eBPF Engine

Documentation for packet filtering in [`crates/net/src/filter/`](../../../crates/net/src/filter).

## Features
- **Netfilter (`firewall.rs`)**: Stateful connection tracking table and rule-based packet drop/allow policies. System call: `sys_netfilter` (Syscall 50).
- **In-Kernel eBPF Engine**: Bytecode instruction interpreter executing filter routines on incoming raw frames. System call: `sys_bpf` (Syscall 41).
