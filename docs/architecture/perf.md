# Hardware Performance Counters & PMU Engine

This document details CPU Performance Monitoring Unit (PMU) event counting and hardware performance system calls in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides CPU hardware performance event monitoring ([perf.rs](../../kernel/src/arch/perf.rs)) accessing hardware MSR performance counters.

---

## 2. Event Types

| Type | Constant | Event Description |
| :---: | :--- | :--- |
| `0` | `PERF_COUNT_HW_CPU_CYCLES` | Core CPU clock cycles executed |
| `1` | `PERF_COUNT_HW_INSTRUCTIONS` | Retired instruction count |
| `2` | `PERF_COUNT_HW_CACHE_MISSES` | L1/L2/L3 hardware cache misses |

---

## 3. System Call Interface

```c
// Syscall 49: Open a hardware PMU event counter
long sys_perf_event_open(uint32_t event_type, uint64_t config, uint64_t pid);
```

---

## 4. Kernel APIs

*   `pub fn sys_perf_event_open(event_type: u32, config: u64, pid: u64) -> Result<u64, &'static str>`: Configures hardware PMU event counter registers.
