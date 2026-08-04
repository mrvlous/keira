# PCIe ECAM/MSI, Asynchronous I/O (io_uring), & NX/KASLR Subsystems

This document details the PCI Express (PCIe) ECAM & MSI/MSI-X driver interface, Asynchronous Kernel I/O (`io_uring`) engine, and Hardware No-Execute (NX/XD Bit) & KASLR memory protection in Keira Kernel.

---

## 1. PCI Express (PCIe) ECAM & MSI/MSI-X Interrupt Subsystem

Keira Kernel provides high-throughput PCI Express hardware access ([pcie.rs](../../kernel/src/io/pcie.rs)):

*   **Enhanced Configuration Access Mechanism (ECAM)**: Maps PCIe configuration space via 64-bit Memory-Mapped I/O (MMIO) at `0xE0000000`, bypassing legacy PCI I/O ports (`0xCF8`/`0xCFC`).
*   **Message Signaled Interrupts (MSI/MSI-X)**: Enables in-band interrupt message delivery directly to CPU Local APIC cores, eliminating legacy IRQ line sharing contentions.

---

## 2. Asynchronous Kernel I/O Engine (`io_uring`)

Implemented in [io_uring.rs](../../kernel/src/ipc/io_uring.rs):

*   **Submission Queue (SQ) & Completion Queue (CQ)**: Zero-copy ring buffers shared between userland processes and the kernel for submission and reaping of asynchronous I/O tasks.
*   **System Call Interface**:
    *   **Syscall 38 (`sys_io_uring_setup`)**: Allocates and maps SQ/CQ ring buffers.
    *   **Syscall 39 (`sys_io_uring_enter`)**: Submits pending I/O operations and reaps completed results without thread blocking.

---

## 3. Hardware No-Execute (NX/XD Bit) & KASLR Memory Protection

Implemented in [vmm.rs](../../kernel/src/mem/vmm.rs):

*   **No-Execute (NX/XD) Page Enforcement**: Bit 63 (`PAGE_NO_EXECUTE`) is enforced on userland data and stack page table entries to prevent buffer overflow code execution vulnerabilities.
*   **Kernel Address Space Layout Randomization (KASLR)**: Calculates randomized virtual base slides (`KASLR_SLIDE_OFFSET`) during early bootstrapping to randomize kernel text memory mapping.
