.. SPDX-License-Identifier: GPL-2.0-only

======================================
Keira Layered Bare-Metal Network Stack
======================================

The ``net`` domain provides a self-contained bare-metal network stack spanning Layer 2 Ethernet through Layer 7 TLS 1.3 and firewall filtering.

Network Stack Architecture
==========================

.. code-block:: text

   graph TD
       App["Application / Sockets"] --> TLS["app/tls.md<br/>TLS 1.3 Engine"]
       App --> DNS["app/dns.md<br/>DNS Client"]
       App --> DHCP["app/dhcp.md<br/>DHCP Client"]
       TLS --> TCP["transport/tcp.md<br/>TCP State Machine"]
       DNS --> UDP["transport/udp.md<br/>UDP Datagrams"]
       TCP --> IP["network/ip.md<br/>IPv4 Protocol & ICMP"]
       UDP --> IP
       IP --> Filter["filter/firewall.md<br/>Stateful Firewall & eBPF"]
       Filter --> Link["link/ethernet.md<br/>Ethernet II & ARP"]
       Link --> Drivers["driver/e1000.md<br/>e1000, RTL8139, VirtIO & Loopback lo"]

Submodule Index
===============

.. list-table::
   :header-rows: 1
   :widths: 35 17 71

   * - Submodule
     - Focus Area
     - Description
   * - ``link/``
     - Data Link Layer
     - Ethernet II framing and ARP cache resolution
   * - ``network/``
     - Network Layer
     - IPv4 packet handling, routing and ICMP echo
   * - ``transport/``
     - Transport Layer
     - UDP datagrams and stateful TCP connection engine
   * - ``app/``
     - Application Layer
     - DHCP network auto-config, DNS resolver and native TLS 1.3
   * - ``socket/``
     - Socket Layer
     - BSD socket descriptor table, handle abstraction and loopback reflection
   * - ``driver/``
     - Network Drivers
     - Intel e1000, Realtek RTL8139, VirtIO-Net and virtual Loopback (``lo``)
   * - ``filter/``
     - Packet Filtering
     - Stateful packet inspection firewall and in-kernel eBPF
