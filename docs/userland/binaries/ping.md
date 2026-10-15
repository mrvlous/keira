<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `ping.elf` Freestanding Core Utility

The `ping.elf` executable (`/bin/ping.elf` and `/bin/ping`) is a freestanding Ring 3 core network utility that tests round-trip network connectivity and latency using ICMP Echo Requests.

---

## 1. Architectural Purpose

Network reachability verification is managed entirely in unprivileged userspace:

1. **Round-Trip Latency Verification**: Measures network responsiveness and latency in milliseconds.
2. **ICMP Protocol Verification**: Validates end-to-end packet transmission through Keira's socket and network drivers.
3. **Loopback & Bare-Metal Testing**: Routes requests transparently to the zero-copy loopback adapter (`127.0.0.1`) or physical Ethernet (`eth0`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-c <count>` | None | Number of echo requests to transmit (default: 4) |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Ping loopback interface with default count:
keira:/# ping 127.0.0.1
PING 127.0.0.1 (127.0.0.1): 56 data bytes
64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0 ms
64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0 ms
64 bytes from 127.0.0.1: icmp_seq=3 ttl=64 time=0 ms
64 bytes from 127.0.0.1: icmp_seq=4 ttl=64 time=0 ms
--- 127.0.0.1 ping statistics ---
4 packets transmitted, 4 packets received, 0% packet loss

# Ping a remote host with specific count:
keira:/# ping -c 2 10.0.2.2
PING 10.0.2.2 (10.0.2.2): 56 data bytes
64 bytes from 10.0.2.2: icmp_seq=1 ttl=64 time=1 ms
64 bytes from 10.0.2.2: icmp_seq=2 ttl=64 time=1 ms
--- 10.0.2.2 ping statistics ---
2 packets transmitted, 2 packets received, 0% packet loss
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/ping/main.c`
- **Binary Locations**: `/bin/ping.elf`, `/bin/ping`
- **System Call**: Vector 89 (`SYS_PING`) via `sys_ping(const char *target, int timeout_ms)`
- **Privilege Level**: Ring 3 unprivileged
