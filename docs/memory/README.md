<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Memory Management Subsystems

The `memory` domain encompasses physical frame management, virtual memory paging, dynamic heap allocation, swap partitions, and DMA buffering.

---

## Memory Architecture

```mermaid
graph TD
    App["Application / Kernel"] --> Heap["heap/<br/>Free-List Allocator & Slab Cache"]
    App --> VMM["vmm/<br/>Virtual Memory Manager & 4-Level Paging"]
    VMM --> PMM["pmm/<br/>Physical Frame Bitmap Allocator"]
    VMM --> Swap["swap/<br/>Disk Paging Engine"]
    Driver["Device Drivers"] --> DMA["dma/<br/>Contiguous DMA Buffers"]
    DMA --> PMM
```

---

## Submodule Index

| Submodule | Focus Area | Description |
| :--- | :--- | :--- |
| [`pmm/`](pmm/README.md) | Physical Memory | Bitmap allocator, usable memory regions, physical frames |
| [`vmm/`](vmm/README.md) | Virtual Memory | 4-level/2-level paging, address spaces, page fault handler |
| [`heap/`](heap/README.md) | Kernel Heap | Segregated free-list, size classes, slab object caching |
| [`swap/`](swap/README.md) | Swap Engine | Disk-backed virtual memory paging and page-out daemon |
| [`dma/`](dma/README.md) | DMA Buffering | Physically contiguous, non-cached direct memory access buffers |
