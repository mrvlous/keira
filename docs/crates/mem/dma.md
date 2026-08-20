<!-- SPDX-License-Identifier: GPL-2.0-only -->

# DMA Contiguous Buffers & Scatter-Gather

Documentation for DMA allocation in [`crates/mem/src/dma/`](../../../crates/mem/src/dma).

## Features
- Allocates physically contiguous, page-aligned buffers required for Bus Master DMA (AHCI SATA PRDTs, e1000 RX/TX descriptor rings, NVMe Submission/Completion queues).
- Implements `ScatterGatherEntry` list mapping for fragmented buffer transfers.
