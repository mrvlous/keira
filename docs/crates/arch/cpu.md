<!-- SPDX-License-Identifier: GPL-2.0-only -->

# CPU Instructions, Port I/O & MSRs

Documentation for low-level CPU control in [`crates/arch/src/cpu/`](../../../crates/arch/src/cpu).

## Port I/O Instructions
Provides inline assembly wrappers for x86 hardware I/O ports:
- `inb(port: u16) -> u8` / `outb(port: u16, val: u8)`
- `inw(port: u16) -> u16` / `outw(port: u16, val: u16)`
- `inl(port: u16) -> u32` / `outl(port: u16, val: u32)`

## Model Specific Registers (MSRs)
- `read_msr(msr: u32) -> u64`: Executes `rdmsr` instruction.
- `write_msr(msr: u32, value: u64)`: Executes `wrmsr` instruction.

Key MSRs:
- `IA32_EFER` (`0xC0000080`): Extended Feature Enable Register (LME, LMA, SCE).
- `IA32_STAR` (`0xC0000081`): Syscall Target Address Register (Target CS/SS).
- `IA32_LSTAR` (`0xC0000082`): Long Mode Syscall Target RIP (`syscall_entry`).
- `IA32_FMASK` (`0xC0000084`): Syscall Flag Mask Register (clears IF, TF).
