<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Architecture-Specific Runtime Stubs & Linker Scripts

The `arch` submodule defines the lowest-level Ring 3 runtime bootstrap stubs and layout scripts (`userland/arch/`).

---

## Architecture Layout

* **`userland/arch/x86/i686/`**: 32-bit x86 runtime entry point and linker script.
* **`userland/arch/x86/x86_64/`**: 64-bit x86_64 runtime entry point and linker script.

---

## Submodule Documents

| Document | Focus Area | Description |
| :--- | :--- | :--- |
| [`crt0.md`](crt0.md) | CRT0 Entry Stub | Process entry point (`_start`), argument unwinding, and `main()` invocation |
| [`linker.md`](linker.md) | ELF Linker Scripts | Memory layout, section alignment, text/data/bss segments, and entry symbols |
