<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Debugging with GDB, Serial & QEMU

Techniques for debugging low-level kernel routines, page faults, and hardware interrupts.

## Remote GDB Debugging
Launch Keira in QEMU debug mode (freezes CPU execution at boot and listens on TCP port `1234`):
```bash
make debug
```

In a separate terminal, attach GDB:
```bash
gdb build/x86_64/bin/keira.bin
(gdb) target remote localhost:1234
(gdb) break kernel_main
(gdb) continue
```

## COM1 Serial Tracing
All early boot messages and panic dumps are mirrored to COM1 serial (`-serial stdio`). Redirect output to a log file:
```bash
qemu-system-x86_64 -cdrom build/x86_64/iso/keira-x86_64-*.iso -serial file:serial.log -display none
```
