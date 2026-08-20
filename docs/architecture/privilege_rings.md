<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Privilege Rings & Hardware Isolation

Keira Kernel strictly separates execution privileges between Ring 0 (Kernel Mode) and Ring 3 (User Mode).

## Hardware Privilege Mechanisms
1. **Global Descriptor Table (GDT)**:
   - `0x08`: 64-bit Kernel Code Segment (`CS`) - Ring 0
   - `0x10`: Kernel Data Segment (`SS`) - Ring 0
   - `0x18`: 64-bit User Code Segment (`CS`) - Ring 3
   - `0x20`: User Data Segment (`SS`) - Ring 3
   - `0x28`: Task State Segment (`TSS`) descriptor

2. **System Call MSRs**:
   - `IA32_STAR`: Encodes target kernel and user segment selectors.
   - `IA32_LSTAR`: Points directly to `syscall_entry` in assembly.
   - `IA32_FMASK`: Clears `IF` and `TF` flags on syscall entry to prevent interrupt re-entrancy.

3. **Task State Segment (TSS)**:
   - Contains `RSP0` pointer for privilege transition stack switching.
