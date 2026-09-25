<!-- SPDX-License-Identifier: GPL-2.0-only -->

# DMA Buffer Allocation & Coherency

* **Allocation**: Allocates power-of-two physically contiguous frames from the PMM.
* **Cache Management**: Configures page table entries with Write-Combining or Cache-Disable flags to guarantee DMA coherency.
