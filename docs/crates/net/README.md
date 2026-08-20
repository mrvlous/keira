<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-net` - Full-Featured Network Stack

The `keira-net` crate implements NIC hardware drivers, Ethernet, ARP, IPv4, ICMP Ping, UDP, DHCP, DNS resolver, TCP state engine, BSD sockets, native TLS 1.3 cryptographic engine, Netfilter firewall, and in-kernel eBPF packet filters.

## Submodules

- [`drivers.md`](drivers.md): Intel e1000, Realtek RTL8139, and VirtIO-Net drivers.
- [`ethernet.md`](ethernet.md): Ethernet frame encapsulation.
- [`arp.md`](arp.md): Dynamic 16-slot ARP cache.
- [`ip.md`](ip.md): IPv4 packet parsing and checksums.
- [`icmp.md`](icmp.md): ICMP Echo Request/Reply protocol.
- [`udp.md`](udp.md): UDP datagram transmission.
- [`dhcp.md`](dhcp.md): DHCP client auto-configuration.
- [`dns.md`](dns.md): UDP port 53 DNS resolver & cache.
- [`tcp.md`](tcp.md): 3-way handshake TCP state engine.
- [`socket.md`](socket.md): POSIX BSD socket layer.
- [`tls.md`](tls.md): Native TLS 1.3 cryptographic engine.
- [`firewall.md`](firewall.md): Netfilter firewall & in-kernel eBPF engine.
