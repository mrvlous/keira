<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hardware PMU Performance Counters

Documentation for Performance Monitoring Unit counters in [`crates/arch/src/perf/pmu.rs`](../../../crates/arch/src/perf/pmu.rs).

## System Call Interface (`sys_perf_event_open` - Syscall 47)
Allows userland profiling tools and system monitors to configure hardware performance counters:
- CPU Instructions Retired (`PERF_COUNT_HW_INSTRUCTIONS`)
- CPU Clock Cycles (`PERF_COUNT_HW_CPU_CYCLES`)
- Cache Misses (`PERF_COUNT_HW_CACHE_MISSES`)
- Branch Mispredictions (`PERF_COUNT_HW_BRANCH_MISSES`)
