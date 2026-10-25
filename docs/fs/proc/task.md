<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Task Metric Nodes

Exposes per-process directories (`/proc/<pid>/`) with task status, process memory usage and command lines:

* `/proc/<pid>/status`: Human-readable task diagnostic status (`Name`, `State`, `Tgid`, `Pid`, `PPid`, `Uid`, `Gid`, `FDSize`, `VmSize`, `VmRSS`, `voluntary_ctxt_switches`).
* `/proc/<pid>/stat`: Linux-compatible 52-field space-delimited kernel process telemetry (PID, comm, state, ppid, pgrp, session, tty, utime, stime, priority, nice, num_threads, starttime, vsize, rss).
* `/proc/<pid>/statm`: 7-field memory page usage metrics (`size`, `resident`, `shared`, `text`, `lib`, `data`, `dt`).
* `/proc/<pid>/cmdline`: Null-delimited command line arguments executed by the task.
* `/proc/<pid>/mounts`: Per-process VFS mount point table.
* `/proc/self/`: Dynamic symbolic link or virtual alias resolving to the calling process directory.
