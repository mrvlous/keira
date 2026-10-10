<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `hostname.elf` Freestanding Core Utility

The `hostname.elf` executable (`/bin/hostname.elf` and `/bin/hostname`) is a freestanding Ring 3 core utility that queries or sets the system network node hostname.

---

## 1. Architectural Purpose

System hostname management is separated from Ring 0 supervisor space:

1. **Persistent Configuration File**: Reads and updates `/etc/hostname` on the root filesystem.
2. **Standard Output Querying**: When invoked without arguments, displays the active system hostname followed by a newline.
3. **Configuration Mutation**: When provided with a name argument, updates `/etc/hostname` with the new value.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Query active system hostname:
keira:/# hostname
keira

# Set new system hostname:
keira:/# hostname keira-box
Hostname set to 'keira-box'

# Verify updated hostname:
keira:/# hostname
keira-box
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/hostname/main.c`
- **Binary Locations**: `/bin/hostname.elf`, `/bin/hostname`
- **Privilege Level**: Ring 3 unprivileged
- **Configuration Path**: `/etc/hostname`
