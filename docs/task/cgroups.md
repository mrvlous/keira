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
/// Create a new resource control group with memory ceiling and CPU shares.
pub fn create_cgroup(name: &str, max_memory_mb: u64, cpu_shares: u32) -> Result<u32, &'static str>;

/// Query aggregate cgroup subsystem telemetry.
pub fn get_cgroups_stats() -> CgroupsStats;

/// Retrieve slice descriptors by identifier.
pub fn get_cgroup(id: u32) -> Option<CgroupInfo>;

/// Reconfigure memory ceiling and CPU shares on an existing cgroup.
pub fn set_cgroup_limits(id: u32, max_memory_mb: u64, cpu_shares: u32) -> Result<(), &'static str>;

/// Delete a non-root cgroup slice.
pub fn delete_cgroup(id: u32) -> Result<(), &'static str>;
```
