<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Network & Transport Commands

The `net` command suite provides network interface inspection, socket telemetry, and transport protocol utilities.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `network` | `network [subcommand]` | Query network interface cards, MAC, IP configuration, sockets, and ARP routing | `dhcp`: Trigger DHCP lease request<br>`ping <ip>`: Send ICMP echo requests<br>`resolve <domain>`: Query DNS A-record via UDP 53<br>`dns-cache`: Display DNS cache table<br>`-s, --stats`: Display extended TX/RX packet counters<br>`-a, --arp`: Display ARP neighbor resolution table<br>`-c, --cache`: Display 16-slot DNS cache table<br>`-h, --help`: Show help info |
| `download` | `download <url> <dst>` | Stream network file via HTTP/HTTPS directly into storage | `-h, --help`: Show help info |
| `fetch` | `fetch [options] <url>` | Stream and inspect HTTP/HTTPS responses directly to console or inspect headers | `-I, --head`: Display response metadata only<br>`-v, --verbose`: Display diagnostic transport metadata<br>`-o, --output <file>`: Save payload directly to storage<br>`-h, --help`: Show help info |
| `https` | `https <domain>` | Establish secure TLS 1.3 encrypted handshake and query page | `info`: Query TLS engine parameters<br>`sha256`: Execute FIPS 180-4 SHA-256 self-test<br>`-h, --help`: Show help info |
| `firewall` | `firewall` | Inspect and configure stateful IPv4 packet filtering rules | `-h, --help`: Show help info |
