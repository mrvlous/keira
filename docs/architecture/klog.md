<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Kernel Event Logging Ring Buffer & Syslog Subsystem

This document details the in-memory circular syslog `dmesg` log ring buffer, severity log levels, and `sys_syslog` system call in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel maintains an in-memory circular ring buffer ([klog.rs](../../kernel/src/entry/klog.rs)) to store diagnostic kernel boot events, driver notices, and panic messages.

---

## 2. Severity Log Levels

Kernel log messages are categorized into 8 severity levels:

| Level | Constant | Severity Description |
| :---: | :--- | :--- |
| `0` | `KERN_EMERG` | System is unusable (Kernel Panic) |
| `1` | `KERN_ALERT` | Action must be taken immediately |
| `2` | `KERN_CRIT` | Critical hardware error conditions |
| `3` | `KERN_ERR` | General error conditions |
| `4` | `KERN_WARNING` | Warning messages |
| `5` | `KERN_NOTICE` | Normal but significant conditions |
| `6` | `KERN_INFO` | Informational messages |
| `7` | `KERN_DEBUG` | Low-level debug messages |

---

## 3. System Call Interface

```c
// Syscall 44: Read kernel diagnostic syslog ring buffer
long sys_syslog(char *buf_ptr, unsigned long len);
```

### Kernel APIs

*   `pub fn klog(level: u8, msg: &str)`: Appends a diagnostic log message to the 4096-byte circular ring buffer.
*   `pub fn sys_syslog_read(buf_ptr: *mut u8, len: usize) -> Result<usize, &'static str>`: Copies log history to a userland buffer for consumption by terminal logging tools.
