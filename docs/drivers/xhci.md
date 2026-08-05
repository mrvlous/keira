# USB 3.0 xHCI Host Controller Isochronous Transfer Driver

This document details the USB 3.0 xHCI host controller isochronous transfer driver architecture in Keira Kernel.

## Overview
The xHCI driver extension, implemented in [xhci.rs](../../kernel/src/io/xhci.rs), manages high-speed USB 3.0 isochronous transfer ring buffers via **Syscall 67 (`sys_xhci_iso`)**.

## Architectural Features
*   **Isochronous Ring Descriptors**: Submits periodic, latency-sensitive audio/video stream transfers over xHCI USB 3.0 endpoints.
*   **Transfer Request Blocks (TRB)**: Formats and manages xHCI host controller transfer ring descriptors.

---
*Back to [Architecture Index](../README.md)*
