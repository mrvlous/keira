<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# In-Kernel DNS Resolver and LRU Cache Table

## Overview

The Keira Kernel incorporates a high-performance in-kernel DNS resolver engine coupled with a 16-slot Dynamic Least Recently Used (LRU) cache table to provide instantaneous 0ms domain resolution for repeated queries.

```
+-------------------------------------------------------------+
|               Kernel DNS Resolution Pipeline                |
+-------------------------------------------------------------+
| Domain Query -> Cache Lookup (LRU) -> Hit? Return IP (0ms)  |
|                                    -> Miss? UDP 53 Request  |
| UDP 53 Response -> Parse Answer -> Store in LRU Table       |
+-------------------------------------------------------------+
```

## Architecture & Cache Design

* **Dynamic LRU Cache Table**: Holds up to 16 host-to-IPv4 entries.
* **Hit Counter & Timestamps**: Tracks access frequency and query hits.
* **DNS Query Packet Generator**: Formats RFC 1035 UDP datagrams directed to configured DNS servers (`8.8.8.8` / `1.1.1.1`).
* **Answer Section Parser**: Extracts A-record IPv4 addresses from recursive network responses.

## System Call Vector

* `sys_dns_resolve(hostname, ip_out)`: Resolves a null-terminated hostname string into an IPv4 address buffer.
