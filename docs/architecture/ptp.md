<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX PTP Hardware Clock Subsystem

This document details the IEEE 1588 Precision Time Protocol (PTP) hardware clock architecture in Keira Kernel.

## Overview
The PTP clock subsystem, implemented in [ptp.rs](../../kernel/src/arch/ptp.rs), provides nanosecond-precision hardware clock frequency synchronization via **Syscall 68 (`sys_ptp_clock`)**.

## Architectural Features
*   **IEEE 1588 Synchronization**: Adjusts hardware clock phase and frequency for network precision time protocols.
*   **Nanosecond Resolution**: Serves high-accuracy time queries for distributed real-time systems.

---
*Back to [Architecture Index](../README.md)*
