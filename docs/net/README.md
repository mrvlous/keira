<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Layered Bare-Metal TCP/IP Protocol Stack

The `net` subsystem implements a complete, zero-dependency TCP/IP stack from raw Ethernet frame parsing to socket abstraction, native TLS 1.3, and stateful packet filtering.

---

## Network Stack Protocol Hierarchy

```mermaid
graph TD
    App["Application Layer<br/>(HTTP Download, HTTPS, DNS, DHCP)"] --> Socket["socket.md<br/>BSD Socket Table"]
    Socket --> Transport["Transport Layer<br/>(tcp.md, udp.md)"]
    Socket --> TLS["tls.md<br/>Native TLS 1.3 Engine"]
    Transport --> Network["Network Layer<br/>(ip.md, icmp.md, firewall.md)"]
    Network --> Link["Data Link Layer<br/>(ethernet.md, arp.md)"]
    Link --> Drivers["NIC Drivers<br/>(Intel e1000, Realtek RTL8139)"]
```

---

## Network Protocol Index

| Layer | Protocol | Document | Description |
| :--- | :--- | :--- | :--- |
| **Layer 2** | Ethernet | [`ethernet.md`](ethernet.md) | IEEE 802.3 framing, EtherType dispatching, and MAC parsing |
| **Layer 2.5** | ARP | [`arp.md`](arp.md) | Address Resolution Protocol table cache and query broadcast |
| **Layer 3** | IPv4 | [`ip.md`](ip.md) | IPv4 packet routing, fragment reassembly, and header checksums |
| **Layer 3** | ICMP | [`icmp.md`](icmp.md) | Internet Control Message Protocol Echo Request/Reply (Ping) |
| **Layer 4** | UDP | [`udp.md`](udp.md) | Stateless User Datagram Protocol packet processing |
| **Layer 4** | TCP | [`tcp.md`](tcp.md) | Stateful TCP 3-way handshake, retransmission timers, and windowing |
| **Layer 7** | DHCP | [`dhcp.md`](dhcp.md) | Dynamic Host Configuration Protocol auto-configuration client |
| **Layer 7** | DNS | [`dns.md`](dns.md) | Domain Name System resolver over UDP port 53 |
| **Security** | TLS 1.3 | [`tls.md`](tls.md) | Bare-metal Transport Layer Security 1.3 with AES-128-GCM |
| **Security** | Firewall | [`firewall.md`](firewall.md) | Stateful Netfilter packet filter and iptables rules |
| **API** | Sockets | [`socket.md`](socket.md) | POSIX BSD socket descriptor table and API |
