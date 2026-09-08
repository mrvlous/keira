<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process & Task Control Shell Commands

This document details all native commands in Keira Kernel related to task monitoring, scheduling, signal dispatching, job control, cgroups, and futex synchronizations.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `tasks` | `tasks` | `[Active]` | Display running kernel and userland tasks, PIDs, states, and CPU runtime |
| `run` | `run <elf_path> [args...]` | `[Active]` | Load and execute an ELF binary in isolated Ring 3 userland address space |
| `kcc` | `kcc [options] <source.c>` | `[Active]` | Compile C source code into a freestanding Ring 3 ELF binary on-demand |
| `stop` | `stop <pid>` | `[Active]` | Pause task execution in scheduler (`SIGSTOP`) |
| `kill` | `kill <pid> [sig]` | `[Active]` | Send POSIX signal (`SIGKILL`, `SIGTERM`, `SIGINT`) to a target task |
| `jobs` | `jobs` | `[Active]` | List active background and stopped shell job entries in scheduler table |
| `fg` | `fg <job_id>` | `[Active]` | Bring background job to foreground and hook keyboard input |
| `bg` | `bg <job_id>` | `[Active]` | Resume stopped job in the background (`SIGCONT`) |
| `cgroups` | `cgroups <status \| list \| create \| set \| delete>` | `[Active]` | Inspect process cgroups resource quotas, memory limits, and CPU shares |
| `futex` | `futex <status \| list \| wait \| wake \| requeue \| reset>` | `[Active]` | Manage Fast Userspace Mutex wait queues and hash tables (Syscall 32 & 40) |
| `eventfd` | `eventfd <status \| list \| create \| read \| write \| close>` | `[Active]` | Manage EventFD 64-bit event notification counter descriptors (Syscall 50 & 51) |
| `perf` | `perf <status \| stat \| top \| reset>` | `[Active]` | Query Hardware Performance Monitoring Counters and CPU profiling (Syscall 49 & 77) |
| `timer` | `timer <status \| list \| create \| cancel>` | `[Active]` | Manage POSIX High-Resolution Interval Timers (Syscall 45 & 46) |

---

## Detailed Usage

### `tasks`
Displays the real-time preemptive scheduler runqueue:
```bash
keira> tasks
  PID  PPID  STATE     PRIO  NAME          MEM (KB)  CPU TIME
  0    0     RUNNING   0     [idle]        0         1240 ms
  1    0     SLEEPING  10    [init]        64        15 ms
  2    1     RUNNING   20    [shell]       128       450 ms
  3    2     RUNNING   15    kcc.elf       512       80 ms
```

### `run <elf_path>`
Loads and executes a dynamic ELF binary within an isolated virtual address space:
```bash
keira> run /apps/bin/calc.elf
```

### `kcc [options] <source.c>`
Compiles C source code on-demand into an executable ELF binary:
```bash
keira> kcc /apps/src/calc.c -o /apps/bin/calc.elf
Compiling: /apps/src/calc.c -> /apps/bin/calc.elf
[OK] Executable ready at /apps/bin/calc.elf
Hint: Execute with 'run /apps/bin/calc.elf'
```

### `cgroups <subcommand>`
Manages resource control groups, memory limits, and CPU shares:
```bash
keira> cgroups status
Resource Control Groups (cgroups) Subsystem Status:
  Subsystem Engine : Active (Memory Controller & Proportional CPU Shares)
  Active Groups    : 3 / 8 slices configured
  Total Used Memory: 14 MB
  Total Max Memory : 112 MB configured ceiling
  PID Namespaces   : Isolated Container Namespaces Mapped

keira> cgroups list
ID   NAME             USED_MEM   MAX_MEM    CPU_SHARES  TASKS
0    root             8 MB       64 MB      1024        3
1    system.slice     2 MB       16 MB      512         2
2    user.slice       4 MB       32 MB      512         1

keira> cgroups create app.slice 24 256
[OK] Created cgroup 'app.slice' (ID #3, Max Mem: 24 MB, CPU Shares: 256)
```

### `futex <subcommand>`
Inspects and manages in-kernel Fast Userspace Mutex wait queues:
```bash
keira> futex status
Fast Userspace Mutex (Futex) Subsystem Status:
  Subsystem Engine : Active (In-Kernel Wait Queues & Hash Table)
  Active Waiters   : 1 / 16 slots in use
  Total Waits      : 1 ops
  Total Wakes      : 0 ops
  Total Requeues   : 0 ops
  Syscall Vectors  : Syscall 32 (futex) / Syscall 40

keira> futex list
SLOT  UADDR        PID   EXPECTED_VAL  BITSET
0     0x00400000   1     1             0xFFFFFFFF

keira> futex wake 0x400000 1
[OK] Woke 1 waiter(s) at address 0x00400000
```

### `eventfd <subcommand>`
Allocates, inspects, and dispatches 64-bit event notification counters:
```bash
keira> eventfd status
EventFD & SignalFD Subsystem Status:
  Subsystem Engine : Active (In-Kernel 64-bit Counter Descriptors)
  Active Descriptors: 1 / 16 allocated
  Total Writes     : 1 ops
  Total Reads      : 0 ops
  Syscall Vectors  : Syscall 50 (eventfd) / Syscall 51 (signalfd)

keira> eventfd list
ID   COUNTER              FLAGS
0    1                    0x00000000

keira> eventfd write 0 5
[OK] Incremented EventFD #0 by 5

keira> eventfd read 0
[OK] Read from EventFD #0: Counter = 6 (Reset to 0)
```

### `perf <subcommand>`
Samples hardware Performance Monitoring Unit (PMU) counters and execution profiling:
```bash
keira> perf status
Hardware Performance Monitoring Unit (PMU) Status:
  Subsystem Engine : Active (Hardware TSC & Architectural PMU Counters)
  PMU Hardware     : ENABLED (Architectural Events Online)
  Nominal Core Hz  : 2400 MHz (2.4 GHz Reference)
  Total TSC Cycles : 3829104 cycles
  Syscall Vectors  : Syscall 49 (perf_event_open) / Syscall 77 (sys_perf_event)

keira> perf stat
Performance Counter Statistics:

  CPU Cycles             : 3840212 cycles
  Instructions Retired   : 4800265 insn (IPC: 1.25)
  L1/L2 Cache Misses     : 37502 misses
  Branch Mispredictions  : 18751 misses

[OK] Execution profiling metrics sampled from Ring 0 PMU counters.
```

### `timer <subcommand>`
Configures and queries POSIX high-resolution interval timers:
```bash
keira> timer status
POSIX High-Resolution Timer Subsystem Status:
  Subsystem Engine : Active (Hardware APIC / LAPIC Tick Driver)
  Clock Sources    : CLOCK_MONOTONIC (1), CLOCK_REALTIME (0)
  Active Timers    : 1 / 8 allocated
  Total Expirations: 250 interrupts serviced
  Syscall Vectors  : Syscall 45 (timer_create) / Syscall 46 (timer_settime)

keira> timer list
ID   CLOCK        INTERVAL_MS  OVERRUNS  STATUS
1    MONOTONIC    10 ms        0         ARMED

keira> timer create 50
[OK] Created POSIX timer #2 (Interval: 50 ms, Clock: MONOTONIC)
```
