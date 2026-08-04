# High-Resolution POSIX Interval Timers Engine

This document details nanosecond interval timer handling, `CLOCK_REALTIME` / `CLOCK_MONOTONIC`, and timer system calls in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements POSIX High-Resolution Interval Timers ([timer.rs](../../kernel/src/arch/timer.rs)) backed by the HPET nanosecond hardware timer.

---

## 2. System Call Interface

```c
// Syscall 45: Create a new POSIX interval timer
long sys_timer_create(uint64_t clock_id, uint64_t *timer_id_ptr);

// Syscall 46: Set interval timeout for an active timer
long sys_timer_settime(uint64_t timer_id, uint32_t flags, uint64_t interval_nanos);
```

---

## 3. Kernel APIs

*   `pub fn sys_timer_create(clock_id: u64, timer_id_ptr: *mut u64) -> Result<u64, &'static str>`: Allocates a new timer handle.
*   `pub fn sys_timer_settime(timer_id: u64, flags: u32, interval_nanos: u64) -> Result<u64, &'static str>`: Configures nanosecond timer intervals.
