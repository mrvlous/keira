<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Service Controller (`ksvc`) & Background Daemon Subsystem

Keira Kernel features a native background service and daemon controller (`ksvc`) accessible via the `service` (or `ksvc`) command. The subsystem manages background workers, periodic daemons, network servers, and configuration files (`.conf`) stored in the canonical `/config/sys/` directory.

---

## 1. Overview & Architecture

Unlike Linux `systemd`, `ksvc` is a lightweight, zero-allocation kernel service supervisor designed specifically for Keira's preemptive scheduler and VFS:

- **Configuration Path**: `/config/sys/*.conf`
- **State Machine**: `RUNNING`, `STOPPED`, `FAILED`
- **Background Loop**: Integrated with `shell::process_pending()` and scheduler ticks.
- **Built-in Services**: `httpd`, `syncd`, `syslogd`, `watchdogd`.

---

## 2. Configuration File Specification (`.conf`)

Service parameter files are stored in `/config/sys/<service>.conf` using a clean key-value syntax:

```ini
# Keira Service Configuration
name=httpd
description=Native Micro Web & REST API Server
enabled=1
port=80
auto_restart=1
```

```ini
# Keira Service Configuration
name=syncd
description=FAT16 Auto-Sync & Cache Flush Daemon
enabled=1
interval=15
auto_restart=1
```

### Supported Parameters:
| Key | Type | Description |
| :--- | :--- | :--- |
| `name` | String (max 16 chars) | Unique service identifier |
| `description` | String (max 48 chars) | Human-readable service purpose |
| `enabled` | Integer (`1` / `0`) | Whether to auto-start on boot |
| `port` | Integer (`1` - `65535`) | Listening TCP port for network services |
| `interval` | Integer (seconds) | Periodic execution cycle interval |
| `auto_restart` | Integer (`1` / `0`) | Restart on error or exit |

---

## 3. CLI Command Reference (`service` / `ksvc`)

```bash
# List all registered background services
service list

# Start a background service
service start httpd

# Inspect detailed service status & telemetry
service status httpd

# Stop a running service
service stop httpd

# Restart service and reload .conf file
service restart httpd

# Enable / Disable auto-start on boot
service enable syncd
service disable watchdogd

# Edit service configuration directly in GNU nano
service edit httpd

# Scaffold a new custom service template
service create custom_worker
```
