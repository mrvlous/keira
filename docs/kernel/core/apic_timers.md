<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Local APIC & Preemption Timers

The Advanced Programmable Interrupt Controller (APIC) provides high-performance interrupt handling and multi-core preemption.

---

## Local APIC Capabilities

* **Base Address**: MSR `0x1B` (`IA32_APIC_BASE`) mapped to virtual memory.
* **Periodic Timer**: Calibrated against the CMOS RTC / HPET to fire at 1000 Hz, driving preemptive task scheduling.
* **Inter-Processor Interrupts (IPIs)**: Used for SMP core bringup and cross-CPU TLB shootdowns.
