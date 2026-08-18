<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Virtio 1.0 Paravirtualized PCI Storage & Network Driver

This document details low-latency Virtio paravirtualized PCI storage and network drivers in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides Virtio 1.0 paravirtualized PCI storage and network drivers ([virtio.rs](../../kernel/src/io/virtio.rs), **Syscall 60 `sys_virtio`**) utilizing Split and Packed Virtqueues for hypervisor I/O acceleration.

---

## 2. System Call Interface

```c
// Syscall 60: Initialize/Query Virtio 1.0 paravirtualized PCI queue status
long sys_virtio(unsigned int device_id, unsigned int queue_idx);
```

---

## 3. Kernel APIs

*   `pub fn sys_virtio(device_id: u32, queue_idx: u32) -> Result<u64, &'static str>`: Configures Virtqueue descriptors and doorbell MMIO registers.
