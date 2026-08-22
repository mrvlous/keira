<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hardware Abstraction Layer (HAL)

Keira Kernel implements a unified, architecture-independent Hardware Abstraction Layer (`crates/arch/src/hal/`) that decouples high-level kernel subsystems (scheduling, memory management, syscalls, and device drivers) from platform-specific processor mechanics.

## HAL Architecture Overview

```text
crates/arch/
├── src/
│   ├── hal/                  # Architecture-independent trait interfaces
│   │   ├── cpu.rs            # Cpu trait (halt, interrupts, core ID, pause)
│   │   ├── mmu.rs            # Mmu trait (page size, TLB flushes, page table root)
│   │   ├── timer.rs          # Timer trait (ticks, uptime, sleep)
│   │   ├── interrupt.rs      # InterruptController trait (EOI, mask, unmask)
│   │   ├── serial.rs         # SerialPort trait (byte/string debug output)
│   │   └── mod.rs            # HAL re-exports and module definitions
│   ├── cpu/                  # x86_64 CPU instructions, MSRs, and CR registers
│   ├── interrupts/           # x86_64 Dual 8259 PIC, APIC, IDT, and SMP
│   ├── timers/               # x86_64 8253 PIT, HPET, TSC, and RTC
│   ├── power/                # x86_64 ACPI power management
│   ├── perf/                 # x86_64 PMU performance monitoring
│   ├── virt/                 # x86_64 Hardware virtualization (KVM/VMX/SVM)
│   ├── debug/                # x86_64 Stack unwinding and debug registers
│   └── lib.rs                # Unified HAL and platform facade
```

## Core HAL Traits

### 1. `Cpu` Trait (`hal/cpu.rs`)
Provides generic CPU lifecycle and hardware interrupt gating:
* `halt()`: Halts the processor core until the next interrupt arrives.
* `enable_interrupts()` / `disable_interrupts()`: Atomically toggles interrupt flags.
* `interrupts_enabled()`: Queries the current interrupt flag status.
* `cpu_id()`: Returns the executing processor core ID.
* `pause()`: Executes an architecture-specific spinlock yield/pause hint.

### 2. `Mmu` Trait (`hal/mmu.rs`)
Standardizes page table switching and translation lookaside buffer (TLB) management:
* `address_bits()`: Returns virtual address width (e.g. 48-bit for 4-level paging).
* `page_size()`: Returns standard page frame size (4096 bytes).
* `flush_tlb(vaddr)`: Flushes TLB entries for a specific virtual address.
* `flush_tlb_all()`: Flushes the entire TLB across all processors.
* `active_table_root()`: Reads the active root page table physical address (`CR3` on x86_64).
* `switch_table_root(root_phys)`: Switches address spaces.

### 3. `Timer` Trait (`hal/timer.rs`)
Defines standardized tick tracking and monotonic system timekeeping:
* `init(frequency_hz)`: Configures periodic timer interrupt rate.
* `ticks()`: Monotonic hardware timer tick count.
* `uptime_ms()`: Milliseconds elapsed since kernel initialization.
* `sleep_ms(ms)`: High-precision busy wait.

### 4. `InterruptController` Trait (`hal/interrupt.rs`)
Encapsulates interrupt vector routing and End-of-Interrupt acknowledgments:
* `init()`: Programs interrupt controllers (PIC/APIC).
* `send_eoi(irq)`: Acknowledges an interrupt vector.
* `mask_irq(irq)` / `unmask_irq(irq)`: Controls hardware interrupt lines.

### 5. `SerialPort` Trait (`hal/serial.rs`)
Provides early debug logging across serial communication lines:
* `init()`: Configures baud rate and line parameters.
* `write_byte(b)` / `write_str(s)`: Transmits debug messages.
* `read_byte()`: Polling receiver for console input.
