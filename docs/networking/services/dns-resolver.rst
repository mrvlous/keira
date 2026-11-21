.. SPDX-License-Identifier: GPL-2.0-only


Domain Name System (DNS) Resolver Subsystem
===========================================

The Keira Domain Name System (DNS) client subsystem (``crates/net/src/dns/``) provides real-time hostname resolution over UDP port 53. It encodes human-readable domain names into standard DNS wire protocol format, transmits queries through the Intel e1000 Ethernet driver to upstream DNS resolvers, decodes dynamic A-record responses and maintains an in-memory DNS cache table.

1. Architectural Motivation & Pure Kernel Integration
=====================================================

Rather than relying on userspace helper daemons, Keira implements kernel-level DNS resolution primitives directly in Ring 0:

1. **Kernel System Call Interface**: Vector 110 (``SYS_DNS_RESOLVE``) allows unprivileged Ring 3 applications (such as ``fetch.elf`` and ``curl``) to resolve domain names dynamically without embedding heavy third-party resolver libraries.
2. **Standard POSIX C Library Integration**: The userland C library ``gethostbyname()`` function queries ``/etc/hosts`` first for local node overrides, falling back to ``SYS_DNS_RESOLVE`` for live internet resolution.
3. **QEMU Virtual Networking Compatibility**: In virtualized development environments using SLIRP user networking (``10.0.2.0/24``), DNS queries are addressed to ``10.0.2.3:53`` with destination MAC set to the virtual router (``52:54:00:12:35:02``), enabling SLIRP to proxy queries to host upstream DNS servers.

2. DNS Wire Protocol & QNAME Encoding
=====================================

The DNS query generator builds standard 12-byte DNS headers followed by question sections:

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Field
     - Value
     - Description
   * - Transaction ID
     - ``0x1234``
     - Unique 16-bit identifier matched against server responses
   * - Flags
     - ``0x0100``
     - Standard query with Recursion Desired (RD) bit set
   * - Questions
     - ``1``
     - Single query question record
   * - Answer RRs
     - ``0``
     - Zero answers in client query
   * - Authority RRs
     - ``0``
     - Zero authority records
   * - Additional RRs
     - ``0``
     - Zero additional records

Domain names are encoded using length-prefixed label octets terminated by a null byte:

.. code-block:: text

   "icanhazip.com" -> \x0a icanhazip \x03 com \x00

Each question specifies Type ``A`` (IPv4 Host Address, value 1) and Class ``IN`` (Internet, value 1).

3. Frame Transmission & Response Parsing
========================================

The resolver generates a raw UDP datagram wrapped in an IPv4 packet and Ethernet frame:

* **Source Address**: Host IP ``10.0.2.15`` on ephemeral UDP port 53530.
* **Destination Address**: Resolver IP ``10.0.2.3`` on well-known UDP port 53.
* **Checksum Verification**: IPv4 header checksum and UDP checksum are computed using standard one's complement addition.
* **Polling & Reception**: The resolver polls the e1000 receive descriptor ring up to 4000 ms, validating the transaction ID and extracting IPv4 answer addresses (Type 1, Length 4) while skipping question labels and compression pointers (``0xC0``).

4. Dynamic DNS Cache Table (``crates/net/src/dns/cache.rs``)
============================================================

To minimize network latency and prevent redundant queries across repetitive system operations, the kernel maintains an LRU-assisted 16-slot DNS cache table:

* **Cache Hits**: Repeated lookups for cached domains return immediately without generating network traffic.
* **Telemetry**: Tracks cache hits per domain slot accessible via supervisor diagnostic commands and kernel logging.
