<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process & Task Control Commands

The `proc` command suite provides task lifecycle control, job management, CPU accounting, and kernel IPC primitives.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `tasks` | `tasks [-s]` | List process table with PID, state, CPU ticks, and switch counters | `-s, --summary`: Scheduler telemetries<br>`-h, --help`: Usage info |
| `run` | `run <binary>` | Execute an ELF binary from storage or launch background worker | `-h, --help` |
| `stop` | `stop <pid>` | Suspend task execution state | `-h, --help` |
| `kill` | `kill [-sig] <pid>` | Send POSIX signals (SIGTERM, SIGKILL, SIGINT) to process | `-h, --help` |
| `jobs` | `jobs` | Display active background jobs, shell job IDs, and status | `-h, --help` |
| `fg` | `fg <job_id>` | Bring background job into terminal foreground session | `-h, --help` |
| `bg` | `bg <job_id>` | Resume suspended job as a background task | `-h, --help` |
| `cgroups` | `cgroups` | Inspect control group hierarchy, CPU quota, and memory limits | `-h, --help` |
| `futex` | `futex` | Inspect active userland fast mutex wait queues and hash buckets | `-h, --help` |
| `eventfd` | `eventfd` | Query event notification file descriptors and counters | `-h, --help` |
| `epoll` | `epoll` | Display I/O event multiplexer descriptors and registered events | `-h, --help` |
| `mqueue` | `mqueue` | Query POSIX message queue descriptors, capacity, and depth | `-h, --help` |
| `timer` | `timer` | Display active timer file descriptors and alarm triggers | `-h, --help` |
| `kcc` | `kcc <file.c>` | In-kernel C compiler for translating source directly to ELF | `-h, --help` |
