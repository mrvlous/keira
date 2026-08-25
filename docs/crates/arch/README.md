<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-arch` - x86 & x86_64 Architecture Subsystem

The `keira-arch` crate encapsulates CPU assembly instructions, Model Specific Registers (MSRs), APIC/IDT interrupt routing, hardware timers (HPET, PIT), PMU performance monitoring, ACPI power transitions, callstack unwinding, and bare-metal hardware virtualization (KVM) for both 64-bit (`x86_64`) and 32-bit (`i686`) target architectures.

## Submodules

- [`cpu.md`](cpu.md): Low-level port I/O, CPUID, CR0..CR4, and MSR control.
- [`interrupts.md`](interrupts.md): Local APIC, Dual 8259 PIC, and 32-bit / 64-bit IDT registration.
- [`timers.md`](timers.md): HPET, 8253 PIT, and POSIX nanosecond interval timers.
- [`perf.md`](perf.md): Performance Monitoring Unit (PMU) counters.
- [`power.md`](power.md): ACPI S5 shutdown, keyboard reset, and NMI watchdog.
- [`unwind.md`](unwind.md): Callstack frame pointer unwinding (RBP/EBP).
- [`virtualization.md`](virtualization.md): Bare-metal KVM hypervisor (Intel VMX & AMD SVM).
