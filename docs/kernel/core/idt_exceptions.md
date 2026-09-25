<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Interrupt Descriptor Table (IDT) & Hardware Exceptions

The Interrupt Descriptor Table (IDT) maps hardware interrupts, CPU faults, and software system call traps to kernel handlers.

---

## 1. IDT Gate Descriptor Layout

On `x86_64`, each IDT entry is 16 bytes:

```text
 127                                                          96
+---------------------------------------------------------------+
|                           Reserved                            |
+---------------------------------------------------------------+
 95                                                           64
+---------------------------------------------------------------+
|                       Offset 63..32                           |
+---------------------------------------------------------------+
 63       48 47       40 39 38 37 36 35 32 31                 16
+-----------+-----------+--+-----+--+-----+---------------------+
|Offset31:16|Access Byte| 0| Rsv | 0| IST |     Selector        |
+-----------+-----------+--+-----+--+-----+---------------------+
 15                                                            0
+---------------------------------------------------------------+
|                       Offset 15..0                            |
+---------------------------------------------------------------+
```

---

## 2. Standard CPU Exception Vectors (0--31)

| Vector | Mnemonic | Exception Name | Error Code | Action in Keira |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | `#DE` | Divide Error | No | Deliver `SIGFPE` or Kernel Panic |
| `0x01` | `#DB` | Debug Exception | No | GDB Stub Breakpoint |
| `0x02` | `NMI` | Non-Maskable Interrupt | No | Hardware Error / Panic |
| `0x03` | `#BP` | Breakpoint (`INT 3`) | No | GDB Debugger Trap |
| `0x04` | `#OF` | Overflow (`INTO`) | No | Trap |
| `0x05` | `#BR` | BOUND Range Exceeded | No | Deliver `SIGSEGV` |
| `0x06` | `#UD` | Invalid Opcode | No | Deliver `SIGILL` |
| `0x07` | `#NM` | Device Not Available (FPU) | No | Lazy FPU State Restore |
| `0x08` | `#DF` | Double Fault | Yes (0) | Dedicated IST Stack / Panic |
| `0x0D` | `#GP` | General Protection Fault | Yes | Deliver `SIGSEGV` or Panic |
| `0x0E` | `#PF` | Page Fault | Yes | Demand Paging / VMM Handler |

---

## 3. Page Fault (`#PF`) Analysis

When Vector `14` (`#PF`) triggers:
1. The CPU stores the faulting linear memory address in register `CR2`.
2. An error code is pushed onto the stack:
   * **Bit 0 (P)**: `0` = Page not present; `1` = Protection violation.
   * **Bit 1 (W/R)**: `0` = Read operation; `1` = Write operation.
   * **Bit 2 (U/S)**: `0` = Kernel mode; `1` = User mode (Ring 3).
   * **Bit 3 (RSVD)**: `1` = Reserved bit set in page table entry.
   * **Bit 4 (I/D)**: `1` = Instruction fetch fault (NX bit violation).

---

## 4. Interrupt & Exception Telemetry Accounting

The architecture layer actively tracks hardware interrupts and CPU exception dispatch events:

* **Per-Vector IRQ Hit Counters (`IRQ_HIT_COUNTERS[0..256]`)**: Atomic counters updated within `isr_handler()` across all hardware IRQs and software vectors.
* `irq_get_counter(vector: usize) -> u64`: Queries total hits for a given interrupt vector.
* `irq_get_total() -> u64`: Computes cumulative interrupt counts across all vectors.
* `get_cpu_exception_count() -> u64`: Returns total trapped CPU exceptions handled by `exception_dispatcher()`.
