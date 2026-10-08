<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Symmetric Multiprocessing (SMP) IPI & Cross-Core TLB Shootdown

The Keira Kernel inter-processor interrupt (IPI) framework enables low-latency inter-core communication and synchronizes translation lookaside buffer (TLB) state across all online application processors (APs).

---

## 1. Local APIC ICR Register Layout

Inter-processor interrupts are programmed through the Local APIC's Interrupt Command Register (ICR), mapped in the physical APIC MMIO window at base `0xFEE00000`:

| Register | Offset | Bits | Field Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `LAPIC_ICR_LOW_REG` | `0x0300` | 0..7 | **Vector** | Interrupt vector number (0x00..0xFF) |
| | | 8..10 | **Delivery Mode** | `000b`: Fixed, `101b`: INIT, `110b`: Startup (SIPI) |
| | | 11 | **Destination Mode** | `0`: Physical APIC ID, `1`: Logical APIC ID |
| | | 12 | **Delivery Status** | `0`: Idle (ready), `1`: Send Pending |
| | | 14 | **Level** | `0`: De-assert, `1`: Assert |
| | | 15 | **Trigger Mode** | `0`: Edge, `1`: Level |
| | | 18..19 | **Destination Shorthand** | `00b`: No shorthand, `01b`: Self, `10b`: All inc self, `11b`: All excl self |
| `LAPIC_ICR_HIGH_REG` | `0x0310` | 24..31 | **Destination Field** | Target APIC ID when no shorthand is selected |

### Delivery Status Invariant
Hardware specifications mandate that before writing to `LAPIC_ICR_LOW_REG`, software must poll bit 12 until it reads 0. This is enforced by `wait_icr_idle()`:
```rust
pub unsafe fn wait_icr_idle() {
    while (read_reg(LAPIC_ICR_LOW_REG) & (1 << 12)) != 0 {
        core::hint::spin_loop();
    }
}
```

---

## 2. Cross-Core Synchronous Rendezvous Protocol

When memory mappings change, stale translations in remote processor TLBs must be purged synchronously to prevent access violations or data races.

### Rendezvous Sequence

```
Initiator CPU                                  Remote Cores (APs)
    |                                                  |
    |-- 1. Acquire TLB_SHOOTDOWN_LOCK (SpinLock)       |
    |-- 2. Store target address to TLB_TARGET_ADDR     |
    |-- 3. Reset TLB_ACK_COUNTER = 0                   |
    |-- 4. Write ICR (Vector 0xFD, All Excl Self) ---->|
    |                                                  |-- 5. Trap Vector 0xFD (isr_tlb_shootdown)
    |-- 6. Invalidate Local TLB (INVLPG or CR3)        |-- 6. Read address & INVLPG
    |                                                  |-- 7. Send Local APIC EOI
    |                                                  |-- 8. Fetch-Add TLB_ACK_COUNTER (+1)
    |<-- 9. Poll TLB_ACK_COUNTER == remote_cores ------|-- 9. IRET to interrupted task
    |-- 10. Release TLB_SHOOTDOWN_LOCK                 |
```

### Memory Ordering Semantics
- **Target Publication**: `TLB_TARGET_ADDR.store(vaddr, Ordering::Release)` ensures all preceding page table mutations are globally visible in physical memory before remote cores process the IPI.
- **Acknowledgment Arrival**: Remote cores increment the counter with `Ordering::Release`.
- **Initiator Barrier**: The initiator polls `TLB_ACK_COUNTER.load(Ordering::Acquire)`, guaranteeing that remote invalidations have retired before the initiator returns to userland or reclaims the physical frame.

### Deadlock Immunity & Failsafe Threshold
To prevent unrecoverable deadlocks in the event that an application processor is halted (e.g., during hardware failure or power down), the spin-wait loop includes a deterministic spin limit (`10_000_000` cycles):
```rust
let mut spins = 0usize;
while TLB_ACK_COUNTER.load(Ordering::Acquire) < remote_cores {
    core::hint::spin_loop();
    spins += 1;
    if spins > 10_000_000 {
        break;
    }
}
```

---

## 3. IDT Vector Allocation & Assembly Handlers

Vector `0xFD` (253) is reserved exclusively for the TLB shootdown interrupt.

- **IDT Gate**: Configured with Ring 0 privilege and interrupt gate attributes (`0x8E`), ensuring hardware clears the `IF` flag upon entry.
- **64-bit Entry (`arch/x86/x86_64/kernel/isr.asm`)**: Implements `swapgs` checking, preserves all registers, invokes `tlb_shootdown_handler`, and issues `iretq`.
- **32-bit Entry (`arch/x86/i686/kernel/isr.asm`)**: Preserves all 32-bit general-purpose and segment registers, executes `tlb_shootdown_handler`, and issues `iretd`.
