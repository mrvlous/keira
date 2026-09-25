<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Global Descriptor Table (GDT) & Task State Segment (TSS)

The Global Descriptor Table (GDT) and Task State Segment (TSS) provide the foundational hardware privilege boundary separating Ring 0 (Kernel) from Ring 3 (Userland).

---

## 1. Segment Selector Layout

Keira uses a flat memory model where code and data segments cover the entire address space. Access permissions are strictly enforced through paging.

| Selector | Index | Privilege (RPL) | Description |
| :--- | :--- | :--- | :--- |
| `0x00` | 0 | Ring 0 | Null Descriptor |
| `0x08` | 1 | Ring 0 | Kernel Code Segment (`CS`) |
| `0x10` | 2 | Ring 0 | Kernel Data Segment (`DS`, `SS`, `ES`) |
| `0x1B` | 3 | Ring 3 | User Data Segment (`SS`, `DS`, RPL = 3) |
| `0x23` | 4 | Ring 3 | User Code Segment (`CS`, RPL = 3) |
| `0x28` | 5 | Ring 0 | Task State Segment (TSS) Descriptor |

---

## 2. Segment Descriptor Format

Each standard GDT entry is 8 bytes wide:

```text
 63          56 55 54 53 52 51    48 47          40 39        32
+--------------+--+--+--+--+--------+--------------+------------+
|  Base 31:24  | G|DB| L|AV|Lim19:16| Access Byte  | Base 23:16 |
+--------------+--+--+--+--+--------+--------------+------------+
 31                          16 15                             0
+------------------------------+--------------------------------+
|          Base 15:0           |          Limit 15:0            |
+------------------------------+--------------------------------+
```

### Access Byte Fields:
* **P (Present)**: Bit 7 (Must be `1` for valid segments).
* **DPL (Descriptor Privilege Level)**: Bits 6:5 (`00` for Kernel, `11` for Userland).
* **S (Descriptor Type)**: Bit 4 (`1` for code/data, `0` for system segments like TSS).
* **Executable**: Bit 3 (`1` for code segment, `0` for data segment).
* **Direction/Conforming**: Bit 2.
* **Readable/Writable**: Bit 1 (`1` for readable code or writable data).

---

## 3. Task State Segment (TSS) Architecture

The TSS provides hardware-mandated stack switching during privilege level transitions:
* When an interrupt or system call occurs while the CPU is in Ring 3, the CPU automatically reads `RSP0` (or `ESP0`) from the TSS and switches to the trusted kernel stack before pushing registers.
* **Interrupt Stack Table (IST)**: On `x86_64`, the TSS provides 7 independent IST pointers, guaranteeing that critical faults (such as Double Fault `#DF` or NMI) execute on a dedicated, clean stack.
