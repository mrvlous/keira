# io_uring Async Network Socket Polling Engine

This document details the io_uring asynchronous network socket polling architecture in Keira Kernel.

## Overview
The async network engine, implemented in [io_uring_net.rs](../../kernel/src/net/io_uring_net.rs), provides zero-copy async network socket polling and multishot accept/receive ring buffers via **Syscall 66 (`sys_io_uring_net`)**.

## Architectural Features
*   **Zero-Copy Network Polling**: Registers network socket descriptors for asynchronous event notifications without Ring 0 context switch overhead.
*   **Multishot Receive**: Sustains continuous network socket packet reception across submission ring queues.

---
*Back to [Architecture Index](../README.md)*
