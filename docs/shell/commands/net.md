<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Network & Transport Commands

The `net` command suite provides network interface inspection, socket telemetry and transport protocol utilities.

> [!NOTE]
> **Pure Kernel Demarcation**: High-level HTTP and web resource retrieval is handled strictly in Ring 3 userspace via the freestanding binary [`/bin/fetch.elf`](../../userland/binaries/fetch.md). The Ring 0 supervisor console provides low-level link diagnostics (`network`) and packet filtering (`firewall`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `network` | `network [subcommand]` | Query network interface cards, MAC, IP configuration, sockets and ARP routing (alias: `net`) | `dhcp`: Trigger DHCP lease request<br>`ping <ip>`: Send ICMP echo requests<br>`resolve <domain>`: Query DNS A-record via UDP 53<br>`dns-cache`: Display DNS cache table<br>`-s, --stats`: Display extended TX/RX packet counters<br>`-a, --arp`: Display ARP neighbor resolution table<br>`-c, --cache`: Display 16-slot DNS cache table<br>`-h, --help`: Show help info |
| `firewall` | `firewall` | Inspect and configure stateful IPv4 packet filtering rules | `-h, --help`: Show help info |
