<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Dynamic Host Configuration Protocol (DHCP)

This document details the DHCP client auto-configuration engine in Keira Kernel.

---

## 4-Step DHCP DORA Exchange

```mermaid
sequenceDiagram
    participant Client as Keira Kernel
    participant Server as DHCP Server (Gateway)

    Client->>Server: 1. DHCP Discover (Broadcast 255.255.255.255:67)
    Server->>Client: 2. DHCP Offer (Offered IP: 10.0.2.15)
    Client->>Server: 3. DHCP Request (Requesting Offered IP)
    Server->>Client: 4. DHCP ACK (Lease Confirmed, Gateway: 10.0.2.2, DNS: 10.0.2.3)
```

---

## Core API (`crates/net/src/dhcp.rs`)

```rust
pub fn dhcp_auto_configure(mac: &MacAddress) -> Result<DhcpConfig, &'static str>;
```
