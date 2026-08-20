<!-- SPDX-License-Identifier: GPL-2.0-only -->

# ARP Table & Address Resolution

Documentation for ARP resolution in [`crates/net/src/arp/`](../../../crates/net/src/arp).

## Features
- Maintains a 16-slot dynamic IP-to-MAC resolution table (`ARP_CACHE`).
- Automatically handles broadcast `ARP Request` (who-has) and `ARP Reply` (is-at) frames.
- Sends gratuitous ARP announcements on network link initialization.
