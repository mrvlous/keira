<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-mem` - Physical & Virtual Memory Management

The `keira-mem` crate implements physical page frame allocation, 4-level x86_64 paging (VMM), dynamic kernel heap management, DMA scatter-gather buffers, and virtual memory swapping.

## Submodules

- [`pmm.md`](pmm.md): Physical Memory Manager (4KB page bitmap).
- [`vmm.md`](vmm.md): Virtual Memory Manager (PML4, PDPT, PD, PT).
- [`heap.md`](heap.md): Kernel dynamic heap allocator.
- [`dma.md`](dma.md): Physically contiguous DMA buffers.
- [`swap.md`](swap.md): Anonymous page swap space pager.
