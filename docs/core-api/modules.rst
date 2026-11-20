.. SPDX-License-Identifier: GPL-2.0-only


In-Kernel Module Subsystem & Symbol Registry
============================================

Keira provides a Linux-grade modular monolithic architecture in Ring 0. The in-kernel module subsystem manages loadable kernel modules (LKMs), dynamically tracks active device drivers and filesystems in ``MODULE_TABLE`` and resolves symbols via ``kallsyms``.

1. Architectural Architecture & Module Table
--------------------------------------------

The module management subsystem resides in ``crates/core/src/module/``. Kernel modules are represented by the ``KernelModule`` descriptor:

.. code-block:: rust

   pub struct KernelModule {
       pub name: [u8; MAX_MODULE_NAME],
       pub name_len: usize,
       pub size: usize,
       pub state: ModuleState,
       pub ref_count: u32,
       pub load_address: u64,
       pub description: &'static str,
   }

The global registry ``MODULE_TABLE`` holds up to 16 dynamically registered kernel extensions. Initial built-in drivers (such as ``ext4_fs``, ``e1000_nic`` and ``ahci_sata``) are registered at boot with active reference counts and base higher-half addresses.

2. Module Lifecycle States
--------------------------

A loadable kernel module traverses four canonical lifecycle states:

* ``Unloaded``: Slot is vacant or unmapped from kernel virtual memory.
* ``Loading``: Initialization routine is executing and registering resources.
* ``Live``: Module is fully operational and exported symbols are serving requests.
* ``Unloading``: Teardown routine is active, verifying zero reference counts before removal.

3. Ring 0 Supervisor Module Commands
------------------------------------

The supervisor console provides native commands to inspect, insert and delete modules in Ring 0:

* ``lsmod``: Lists all active kernel modules, memory footprints, reference counts, states and load addresses.
* ``insmod <name> [size]``: Dynamically registers and binds a loadable module into ``MODULE_TABLE``.
* ``rmmod <name>``: Verifies ``ref_count == 0`` and safely removes the module from the table.
* ``modinfo <name>``: Displays detailed metadata including file path, author, license and exported symbols.

4. System Call Interfaces & ProcFS
----------------------------------

Userspace programs interface with the module subsystem via standard Linux system call vectors:

* ``sys_init_module`` (Vector 34): Loads and registers a new module into kernel space.
* ``sys_delete_module`` (Vector 35): Unloads a module if it has zero active dependencies.

Additionally, the state of all kernel modules is exported dynamically via the virtual pseudo-filesystem at ``/proc/modules`` and ``/proc/kallsyms``.

5. In-Kernel ELF Relocatable (.ko) Object Linking & Relocations
---------------------------------------------------------------

Keira provides a native freestanding in-kernel dynamic linker for relocatable ELF object files (``.ko``) across both ``x86_64`` and ``i686`` architectures:

* **ELF Parser (``crates/core/src/module/elf.rs``)**: Validates ``ET_REL`` object headers, extracts section headers, resolves section names from ``.shstrtab`` and enumerates symbols from ``.symtab``.
* **Relocation Engine (``crates/core/src/module/reloc.rs``)**: Computes and patches machine relocations in-place:
  - ``R_X86_64_64`` (direct 64-bit ``S + A``)
  - ``R_X86_64_PC32`` (PC-relative 32-bit signed ``S + A - P``)
  - ``R_X86_64_32`` (direct 32-bit unsigned ``S + A``)
  - ``R_386_32`` (direct 32-bit ``S + A``)
  - ``R_386_PC32`` (PC-relative 32-bit ``S + A - P``)
* **Symbol Resolution & GPL Licensing**: Unresolved external symbols are resolved dynamically against ``BASE_KALLSYMS`` and ``DYNAMIC_SYMBOLS``. Symbols marked with ``is_gpl_only`` require modules to declare GPL-compatible licensing.
* **Dynamic Loader (``crates/core/src/module/loader.rs``)**: Coordinates memory slot assignment, section placement, relocation patching and lifecycle execution (``init_module`` and ``cleanup_module``).
