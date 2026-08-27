<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Panic Engine & Stack Unwinding

This document details stack frame unwinding, register state dumping, and emergency crash reporting in Keira Kernel.

---

## Panic Handler (`crates/kernel/src/panic/mod.rs`)

When an unrecoverable kernel error or assertion failure occurs, Rust invokes the kernel panic hook:

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // 1. Disable hardware interrupts immediately
    unsafe { keira_arch::cpu::cli(); }

    // 2. Switch VGA text console to High-Intensity Red on Black
    vga_set_color(0x0C, 0x00);

    // 3. Print structured panic header and location
    vga_print_str("\n!!!!!!!!!!!!!!!! KERNEL PANIC !!!!!!!!!!!!!!!!\n");
    if let Some(loc) = info.location() {
        // Print file and line
    }

    // 4. Perform stack frame unwinding
    unwind_stack_trace();

    // 5. Enter infinite low-power CPU halt loop
    loop {
        unsafe { keira_arch::cpu::hlt(); }
    }
}
```

---

## Dual-Architecture Stack Frame Unwinding

Stack unwinding traverses the linked chain of base pointer frame records (`RBP`/`EBP`):

```
+-------------------+
| Previous RBP/EBP  | <--- Frame Pointer (RBP)
+-------------------+
| Return Address    | (RBP + 8 / EBP + 4)
+-------------------+
| Local Variables   |
+-------------------+
```
