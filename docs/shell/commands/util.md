<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Utility & Navigation Shell Commands

This document details shell utility commands in Keira Kernel.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `help` | `help [topic]` | Display general help or syntax for a specific command |
| `guide` | `guide [topic]` | View interactive kernel architecture and tutorial guides |
| `history` | `history [clear]` | Display command history ring buffer entries |
| `search` | `search <pattern> [path]` | Search text pattern across files |
| `go` | `go <path>` | Fast directory navigation bookmark jump |
| `script` | `script <file>` | Execute batch shell commands from a script file |
| `wait` | `wait <ms>` | Pause shell execution for specified milliseconds |
| `wipe` | `wipe [screen \| history]` | Clear VGA screen buffer or history entries |

---

## Detailed Usage

### `help [topic]`
```bash
keira> help network
Command: network
Usage: network [status | up | down | dhcp]
Description: Displays NIC state or triggers DHCP auto-configuration.
```
