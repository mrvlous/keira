<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Address Space Layout Randomization (KASLR)

This document specifies Kernel Address Space Layout Randomization (KASLR), entropy generation, and slide offset calculation in Keira Kernel.

---

## KASLR Memory Virtualization Architecture

```mermaid
graph TD
    RNG["Entropy Source (RDRAND / RDTSC / TPM)"] --> SlideGen["Generate 2MB-Aligned Random Slide Offset"]
    SlideGen --> PageTable["Adjust Kernel Page Tables (PML4 / PDPT)"]
    PageTable --> RandomKernel["Kernel Base Mapped at (0xFFFF_8000_0000_0000 + Slide)"]
    RandomKernel --> Protect["Mitigate Return-Oriented Programming (ROP) Attacks"]
```

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Entropy Sources** | `RDRAND` instruction, CPU `RDTSC`, TPM 2.0 TRNG | Cryptographically sound entropy pooling |
| **Alignment** | 2 Megabyte Alignment | Aligns with Huge Page virtual mapping boundaries |
| **Virtual Base Space** | `0xFFFF_8000_0000_0000` to `0xFFFF_8000_4000_0000` | 1 GB randomized kernel execution region |

---

## Core API (`crates/mem/src/kaslr/mod.rs`)

```rust
/// Generate random KASLR slide offset during early boot.
pub unsafe fn calculate_kaslr_slide() -> usize;

/// Apply randomized virtual memory offset to kernel master page tables.
pub unsafe fn apply_kaslr_offset(slide: usize);
```
