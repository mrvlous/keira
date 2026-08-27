<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Background Service Daemon Supervisor

This document specifies the background service supervisor managing system daemons in Keira Kernel.

---

## Service Descriptors

Services are defined by simple descriptor structures:
* **Service Name**: Unique daemon identifier (`syslogd`, `dhcpd`, `httpd`).
* **State**: `Active`, `Stopped`, `Restarting`.
* **Execution Interval**: Periodic timer tick wakeups.

---

## Core API (`crates/shell/src/service.rs`)

```rust
pub fn service_list();
pub fn service_start(name: &str) -> Result<(), &'static str>;
pub fn service_stop(name: &str) -> Result<(), &'static str>;
```
