<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 5: Bare-Metal Networking & TLS

This milestone explores building a complete TCP/IP network stack from raw Ethernet frames up to TLS 1.3 encrypted HTTP communication.

---

## Key Achievements

1. **Network Interface Cards**: Intel e1000 and Realtek RTL8139 hardware drivers with circular DMA ring buffers.
2. **Layer 2 & 3 Protocols**: Ethernet II frame framing, ARP cache resolution, IPv4 routing, and ICMP echo.
3. **Layer 4 Transport**: UDP datagrams and stateful TCP connection state machine (`SYN`, `ESTABLISHED`, `FIN`).
4. **Layer 7 & Security**: DHCP client configuration, DNS resolution, and native TLS 1.3 with AES-128-GCM and X25519.
