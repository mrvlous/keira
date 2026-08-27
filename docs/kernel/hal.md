<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hardware Abstraction Layer (HAL)

This document specifies the abstract hardware traits defined in Keira Kernel to ensure clean decoupling between architecture-independent subsystems and low-level CPU operations.

---

## HAL Core Traits

```rust
pub trait Cpu {
    fn halt(&self) -> !;
    fn enable_interrupts(&self);
    fn disable_interrupts(&self);
    fn are_interrupts_enabled(&self) -> bool;
    fn read_cycle_counter(&self) -> u64;
}

pub trait Mmu {
    unsafe fn map_page(&mut self, virt: usize, phys: usize, flags: usize) -> Result<(), &'static str>;
    unsafe fn unmap_page(&mut self, virt: usize) -> Result<(), &'static str>;
    fn translate(&self, virt: usize) -> Option<usize>;
    fn switch_address_space(&mut self, root_table_phys: usize);
}

pub trait InterruptController {
    fn init(&mut self);
    fn enable_irq(&mut self, irq: u8);
    fn disable_irq(&mut self, irq: u8);
    fn send_eoi(&mut self, irq: u8);
}
```
