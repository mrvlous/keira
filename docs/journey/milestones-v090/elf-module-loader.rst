.. SPDX-License-Identifier: GPL-2.0-only


Milestone 63: In-Kernel ELF Relocatable Module (.ko) Loader & Dynamic Relocations
=================================================================================

Milestone 63 advances Keira's Ring 0 pure kernel architecture by introducing a native in-kernel dynamic linker for relocatable ELF object files (``.ko``). Following the establishment of the module descriptor registry and ``kallsyms`` in Milestone 62, this milestone enables the kernel to parse, link, relocate and execute ELF relocatable binaries directly within Ring 0 memory without relying on userspace dynamic linkers or pre-linked monolithic images.

1. Architectural Motivation & Pure Kernel Dynamic Linking
=========================================================

In a freestanding monolithic kernel modeled after Linux, modularity must be achieved through dynamic in-kernel object linking:

1. **Relocatable ELF Files (``ET_REL``)**: Kernel modules are compiled as standalone object files without fixed memory addresses. The kernel itself must parse section headers, allocate kernel memory frames and resolve inter-section references.
2. **Architectural Relocation Engines**: The kernel must compute and patch machine instruction targets directly in memory for both 64-bit (``x86_64``) and 32-bit (``i686``) architectures.
3. **Symbol Table Binding (``kallsyms``)**: Unresolved external references within the module must be bound dynamically against exported kernel symbols (``BASE_KALLSYMS`` and ``DYNAMIC_SYMBOLS``), with GPL licensing boundaries strictly enforced.
4. **Lifecycle Hooks**: Once relocated, the kernel locates and invokes the module's ``init_module`` entry point, transitioning its operational state to ``Live``.

2. In-Kernel ELF Parser Architecture (``crates/core/src/module/elf.rs``)
========================================================================

The freestanding ELF parser provides zero-allocation parsing for relocatable objects:

* **ELF Header Validation**: Validates magic constant ``\x7fELF``, target class (``ELFCLASS64`` or ``ELFCLASS32``), little-endian encoding and object type ``ET_REL`` (1).
* **Section Header Extraction**: Enumerates section headers, identifying allocatable code (``.text``), read-only data (``.rodata``), initialized data (``.data``) and zero-initialized bss (``.bss``).
* **String & Symbol Tables**: Extracts ``.shstrtab`` for section naming, ``.strtab`` for symbol naming and ``.symtab`` for symbol descriptors.
* **Relocation Table Discovery**: Parses both explicit addend tables (``.rela.*`` with ``SHT_RELA``) and implicit addend tables (``.rel.*`` with ``SHT_REL``).

3. In-Kernel Dynamic Relocation Engine (``crates/core/src/module/reloc.rs``)
============================================================================

The relocation engine calculates target patch values and writes them to module memory:

.. list-table:: Supported In-Kernel Relocation Types
   :widths: 20 20 60
   :header-rows: 1

   * - Architecture
     - Relocation Type
     - Mathematical Calculation & Description
   * - ``x86_64``
     - ``R_X86_64_64``
     - ``S + A``: Direct 64-bit absolute virtual address.
   * - ``x86_64``
     - ``R_X86_64_PC32``
     - ``S + A - P``: PC-relative 32-bit signed displacement.
   * - ``x86_64``
     - ``R_X86_64_32``
     - ``(S + A) as u32``: Direct 32-bit zero-extended address.
   * - ``x86_64``
     - ``R_X86_64_32S``
     - ``(S + A) as i32``: Direct 32-bit sign-extended address.
   * - ``i686``
     - ``R_386_32``
     - ``S + A``: Direct 32-bit absolute address.
   * - ``i686``
     - ``R_386_PC32``
     - ``S + A - P``: PC-relative 32-bit displacement.

4. Dynamic Loader & Memory Allocation (``crates/core/src/module/loader.rs``)
============================================================================

The dynamic loader manages the entire in-memory linking pipeline:

1. **Memory Reservation**: Assigns an aligned memory slot from the higher-half module memory pool (``0xFFFF_8000_0050_0000`` on ``x86_64`` and ``0xC050_0000`` on ``i686``).
2. **Section Copy & Zeroing**: Copies allocatable section data into the slot buffer and zero-fills uninitialized ``.bss`` sections.
3. **Symbol Resolution & GPL Enforcement**: Internal symbols are resolved to their mapped section offsets. External symbols are looked up via ``resolve_symbol()``. If an external symbol is marked ``is_gpl_only`` and the module does not declare a GPL-compatible license, loading is aborted with ``KernelError::PermissionDenied``.
4. **Relocation Application**: All relocation records are processed and patched in-place.
5. **Entry Point Discovery**: Locates ``init_module`` and ``cleanup_module`` hooks and registers exported dynamic symbols into ``DYNAMIC_SYMBOLS``.

5. Supervisor Console & System Call Integration
===============================================

* **Supervisor Commands**: ``insmod`` in ``crates/shell/src/cmds/dev.rs`` now accepts both raw file paths (e.g. ``insmod /lib/modules/driver.ko``) and virtual driver names. ``rmmod`` automatically coordinates ELF teardown.
* **System Call Vectors**: System call vector 34 (``sys_init_module``) detects whether the user passed an in-memory ELF image buffer or a module name string, delegating to the dynamic linker accordingly.

6. Conformance Testing & Verification
=====================================

The implementation was validated through automated tests:

* **Unit Test Suite**: 27 unit tests in ``keira-core`` verifying ELF header decoding, relocation calculations and synthetic module linking.
* **Userland Conformance**: ``/bin/test_modules.elf`` validates both name-based registration and in-memory ELF relocatable object linking via ``sys_init_module``.
