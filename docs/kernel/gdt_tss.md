<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Global Descriptor Table (GDT) & Task State Segment (TSS)

This document describes segment descriptors, privilege level transitions, and Task State Segment stack initialization in Keira Kernel.

---

## GDT Layout & Selectors

Keira Kernel configures a symmetrical Global Descriptor Table covering both Kernel (Ring 0) and User (Ring 3) address spaces:

| Selector | Index | Privilege | Type | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 0 | - | Null | Mandatory CPU Null Descriptor |
| `0x08` | 1 | Ring 0 | Code | 64-bit Kernel Code Segment (R/X) |
| `0x10` | 2 | Ring 0 | Data | Kernel Data Segment (R/W) |
| `0x18` | 3 | Ring 3 | Data | Userland Data Segment (R/W, DPL=3) |
| `0x20` | 4 | Ring 3 | Code | 64-bit Userland Code Segment (R/X, DPL=3) |
| `0x28` | 5 | Ring 0 | TSS | Task State Segment Descriptor (16-byte on x86_64) |

---

## Task State Segment (TSS) Structure

The TSS provides the CPU with a guaranteed, clean stack pointer when transitioning from unprivileged Ring 3 to Ring 0 upon interrupts, exceptions, or system calls.

```rust
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved_0: u32,
    pub rsp0: u64,         // Ring 0 Stack Pointer loaded on privilege escalation
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved_1: u64,
    pub ist1: u64,         // Interrupt Stack Table 1 (Double Fault #DF)
    pub ist2: u64,         // Interrupt Stack Table 2 (NMI)
    pub ist3: u64,         // Interrupt Stack Table 3 (Page Fault #PF)
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved_2: u64,
    pub reserved_3: u16,
    pub iomap_base: u16,
}
```

---

## TSS Initialization Routine (`crates/syscall/src/tss/mod.rs`)

```rust
pub unsafe fn init_user_mode() -> Result<(), &'static str> {
    let tss_addr = &raw const TSS as usize;
    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;

    let stack_frame = pmm::alloc_frame().ok_or("TSS Init: Out of memory for kernel privilege stack")?;
    TSS.rsp0 = stack_frame + pmm::PAGE_SIZE;

    // Populate TSS descriptor in GDT
    reload_gdt();
    load_tss();
    init_syscall_msrs();

    Ok(())
}
```
