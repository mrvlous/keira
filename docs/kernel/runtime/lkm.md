<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Loadable Kernel Modules (LKM)

Keira supports dynamic kernel extension through Loadable Kernel Modules.

---

## Module Architecture

* **Format**: Standard ELF relocatable objects (`.ko` / `.elf`).
* **Symbol Resolution**: Exported kernel symbols are maintained in a global symbol table (`crates/core/src/module/`).
* **Lifecycle**: Modules export `init_module()` and `cleanup_module()` entry points.
