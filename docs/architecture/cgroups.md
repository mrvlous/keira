<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Resource Control Groups (cgroups) & PID Namespaces

This document details in-kernel process resource accounting, memory limits, and container PID namespace isolation in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides Resource Control Groups ([cgroup.rs](../../kernel/src/task/cgroup.rs)) for bounding process memory consumption and isolating process identifier trees across container namespaces.

---

## 2. Kernel Data Structures

```rust
pub struct CGroup {
    pub name: [u8; 32],
    pub memory_limit: usize,
    pub memory_usage: usize,
    pub pids: [u32; 64],
    pub pid_count: usize,
}
```

---

## 3. Kernel APIs

*   `pub fn init()`: Initializes root cgroup structure and default limits.
*   `pub fn add_process(pid: u32) -> Result<(), &'static str>`: Binds a task PID to the active resource control group.
