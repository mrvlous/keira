<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Control Groups (Cgroups) & Resource Limits

This document specifies resource management and isolation policies implemented in Keira Kernel.

---

## Cgroup Resource Parameters

| Subsystem | Parameter | Description |
| :--- | :--- | :--- |
| `cpu` | `cpu.shares` | Relative weight for proportional CPU scheduling (`1024` = Baseline) |
| `cpu` | `cpu.cfs_quota_us` | Max execution time in microseconds per period |
| `memory` | `memory.limit_in_bytes` | Hard maximum physical frame ceiling for the group |
| `memory` | `memory.usage_in_bytes` | Current consumed physical memory in bytes |

---

## Core API (`crates/task/src/cgroups/mod.rs`)

```rust
pub fn cgroup_create(name: &str) -> Result<u32, &'static str>;
pub fn cgroup_attach_task(cg_id: u32, pid: u32) -> Result<(), &'static str>;
pub fn cgroup_set_cpu_limit(cg_id: u32, shares: u32) -> Result<(), &'static str>;
pub fn cgroup_set_memory_limit(cg_id: u32, max_bytes: usize) -> Result<(), &'static str>;
```
