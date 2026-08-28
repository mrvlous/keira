<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Service Controller (`ksvc`) & Background Daemons

This document specifies the internal architecture, configuration management, and runtime execution of background daemon services managed by `ksvc` in Keira Kernel.

---

## Service Controller Architecture

```mermaid
graph TD
    KernelBoot["Kernel Boot / run_boot_script()"] --> InitSvc["auto_start_enabled_services()"]
    InitSvc --> ConfReader["Parse /config/sys/*.conf"]
    ConfReader --> SvcTable["SERVICES Table (MAX_SERVICES = 8)"]
    ShellRunloop["Shell Event Loop (tick_all)"] --> Dispatch["Interval Dispatcher & Socket Poller"]
    Dispatch --> Httpd["httpd: Micro Web & REST API Server (Port 80)"]
    Dispatch --> Syncd["syncd: FAT16 Auto-Sync & Cache Flush (15s)"]
    Dispatch --> Syslogd["syslogd: Kernel Audit Logger (/data/log/syslog.log)"]
    Dispatch --> Watchdogd["watchdogd: Memory & Task Supervisor (10s)"]
```

---

## Managed Built-In Services

| Service Name | Description | Default Port / Interval | Config File Path | Default State |
| :--- | :--- | :--- | :--- | :--- |
| **`httpd`** | Native Micro Web & REST API Server | Port 80 (TCP) | `/config/sys/httpd.conf` | Enabled |
| **`syncd`** | FAT16 Auto-Sync & Dirty Cache Flush | Interval 15s | `/config/sys/syncd.conf` | Enabled |
| **`syslogd`** | Kernel Event & Audit Logger Service | Interval 5s | `/config/sys/syslogd.conf` | Enabled |
| **`watchdogd`** | Memory & Task Health Supervisor | Interval 10s | `/config/sys/watchdogd.conf` | Disabled |

---

## Configuration File Format (`.conf`)

Service configurations are serialized to `/config/sys/<service>.conf`:
```text
# Keira Service Configuration
name=httpd
description=Native Micro Web & REST API Server
enabled=1
auto_restart=1
port=80
```

---

## Core API (`crates/shell/src/service/mod.rs`)

```rust
pub unsafe fn init();
pub unsafe fn auto_start_enabled_services();
pub unsafe fn start_service(name: &str) -> Result<(), &'static str>;
pub unsafe fn stop_service(name: &str) -> Result<(), &'static str>;
pub unsafe fn restart_service(name: &str) -> Result<(), &'static str>;
pub unsafe fn enable_service(name: &str, enable: bool) -> Result<(), &'static str>;
pub unsafe fn tick_all();
```

---

## Interactive Shell Usage

```bash
# List all registered services and their real-time state
keira> ksvc list

# Inspect detailed telemetry of a service
keira> ksvc status httpd

# Start, stop, or restart a background service
keira> ksvc start watchdogd
keira> ksvc stop syncd
keira> ksvc restart httpd

# Enable or disable service boot auto-start
keira> ksvc enable watchdogd
keira> ksvc disable httpd

# View live service logs
keira> ksvc logs syslogd
```
