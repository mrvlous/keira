# Hyper-V Hypercall & Synthetic Interrupt Controller (SynIC) Engine

This document details the Hyper-V hypervisor guest support and SynIC synthetic interrupt controller architecture in Keira Kernel.

## Overview
The Hyper-V engine, implemented in [hyperv.rs](../../kernel/src/arch/hyperv.rs), enables guest VM hypercalls and synthetic interrupt page routing on Microsoft Hyper-V / Azure hypervisors via **Syscall 65 (`sys_hyperv`)**.

## Architectural Features
*   **Synthetic Interrupt Page**: Maps Hyper-V SynIC message/event pages for inter-partition communications.
*   **Hypercall Gateway**: Formats and executes hypercall control codes across guest partitions.

---
*Back to [Architecture Index](../README.md)*
