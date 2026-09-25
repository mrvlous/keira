<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Virtual Machine (KVM) Virtualization

Keira provides initial hooks for hardware-assisted CPU virtualization (Intel VT-x / AMD-V).

---

## Capabilities

* Detection of `VMX` / `SVM` CPUID feature flags.
* Allocation of Virtual Machine Control Structures (VMCS).
* Foundation for nested lightweight guest execution.
