<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Diagnostic Logging (`klog`)

Documentation for the circular in-memory syslog dmesg buffer in [`crates/core/src/log/klog.rs`](../../../crates/core/src/log/klog.rs).

## Severity Levels

| Macro / Constant | Severity | Description |
| :--- | :--- | :--- |
| `KERN_EMERG` | `0` | System is unusable (Kernel Panic) |
| `KERN_ALERT` | `1` | Action must be taken immediately |
| `KERN_CRIT` | `2` | Critical conditions |
| `KERN_ERR` | `3` | Error conditions |
| `KERN_WARNING` | `4` | Warning conditions |
| `KERN_NOTICE` | `5` | Normal but significant condition |
| `KERN_INFO` | `6` | Informational boot and driver notices |
| `KERN_DEBUG` | `7` | Debug-level messages |

## Circular Buffer Architecture
The buffer is structured as a 4096-byte circular ring buffer (`KLOG_RING_BUFFER`). Diagnostic messages can be read via system call 44 (`sys_syslog`).
