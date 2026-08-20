<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-arch` - x86_64 Architecture Subsystem

The `keira-arch` crate encapsulates CPU assembly instructions, Model Specific Registers (MSRs), APIC/IDT interrupt routing, hardware timers (HPET, PIT), PMU performance monitoring, ACPI power transitions, callstack unwinding, and bare-metal hardware virtualization (KVM).

## Submodules

- [`cpu.md`](cpu.md): Low-level port I/O, CPUID, and MSR control.
- [`interrupts.md`](interrupts.md): Local APIC, I/O APIC, and IDT registration.
- [`timers.md`](timers.md): HPET, 8253 PIT, and POSIX nanosecond interval timers.
- [`perf.md`](perf.md): Performance Monitoring Unit (PMU) counters.
- [`power.md`](power.md): ACPI S5 shutdown, keyboard reset, and NMI watchdog.
- [`unwind.md`](unwind.md): Callstack frame pointer unwinding.
- [`virtualization.md`](virtualization.md): Bare-metal KVM hypervisor (Intel VMX & AMD SVM).
