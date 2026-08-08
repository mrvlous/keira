<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX Sched_Autogroup Task Isolation Engine

This document details the POSIX Sched_Autogroup task isolation architecture in Keira Kernel.

## Overview
The autogroup scheduler engine, implemented in [autogroup.rs](../../kernel/src/task/autogroup.rs), automatically groups process tasks per TTY terminal session for CPU time slice fairness via **Syscall 70 (`sys_sched_autogroup`)**.

## Architectural Features
*   **Per-TTY Task Grouping**: Groups child processes spawned under virtual terminals to prevent single-session CPU monopolization.
*   **Hierarchical Scheduling**: Integrates with CFS and EDF schedulers for task group load balancing.

---
*Back to [Architecture Index](../README.md)*
