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
| `cgroups` | `cgroups [list \| status]` | `[Preview]` | Inspect process cgroups resource quotas and PID namespace mapping |
| `futex` | `futex [status]` | `[Preview]` | Inspect Fast Userspace Mutex wait queues and locking interface (Syscall 40) |
| `eventfd` | `eventfd [status]` | `[Preview]` | Inspect EventFD & SignalFD event notification counter interface (Syscall 50 & 51) |
| `perf` | `perf [stat \| top \| list]` | `[Preview]` | Query Hardware Performance Monitoring Counters interface (Syscall 47 & 48) |
| `timer` | `timer [status \| list]` | `[Preview]` | Query POSIX High-Resolution Timer interface (Syscall 45 & 46) |

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
