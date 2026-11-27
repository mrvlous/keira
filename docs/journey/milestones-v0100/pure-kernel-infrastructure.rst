.. SPDX-License-Identifier: GPL-2.0-only


Milestone 62: Pure Kernel Infrastructure, Ring 0 Kernel Threads & In-Kernel Module Management
=============================================================================================

Milestone 62 deepens Keira's architectural identity as a pure freestanding monolithic kernel in Ring 0, aligning its in-kernel subsystem foundations directly with Linux kernel architecture. This milestone introduces native Ring 0 kernel threads (``kthreads``), background worker daemons (``[kthreadd]``, ``[kswapd0]`` and ``[kflushd]``), in-kernel loadable module management (``lsmod``, ``insmod``, ``rmmod`` and ``modinfo``), ProcFS kernel introspection interfaces (``/proc/modules`` and ``/proc/kallsyms``) and reference userland utilities.

1. Architectural Motivation & Pure Kernel Principles
====================================================

Keira is fundamentally a bare-metal monolithic kernel written from first principles in Ring 0. Userland (Ring 3) is strictly an optional reference payload. Monolithic kernels like Linux manage all critical system activities inside Ring 0:

1. **Kernel Threads**: Background tasks must execute autonomously in Ring 0 with kernel page table mappings and kernel stacks, without relying on userspace runtimes or synthetic loops.
2. **Modular In-Kernel Extensions**: The kernel must provide dynamic tracking and lifecycle control for device drivers and filesystems in kernel memory (``MODULE_TABLE``).
3. **Telemetry & Introspection**: Kernel state must be exposed transparently through standardized ProcFS streams (``/proc/modules`` and ``/proc/kallsyms``).

Milestone 62 fulfills these core architectural requirements across both ``x86_64`` and ``i686``.

2. Ring 0 Kernel Threads (``kthreads``) & Daemon Architecture
=============================================================

The kernel thread engine is established in ``crates/task/src/scheduler/kthread.rs``. A kernel thread executes strictly within Ring 0 code segment ``0x08`` and data segment ``0x10`` with ``is_user = false``:

* **Creation Primitive**: The ``kthread_create(name, entry)`` function allocates a vacant task descriptor, allocates a dedicated page-backed kernel stack frame and initializes an ``InterruptContext`` without userspace address space cloning.
* **Master Kernel Daemon (``[kthreadd]``, PID 1)**: Serves as the structural anchor for all deferred background kernel threads.
* **Memory Reclamation Daemon (``[kswapd0]``, PID 2)**: Monitors physical memory pressure, page allocation thresholds and slab caches.
* **Buffer Sync Daemon (``[kflushd]``, PID 3)**: Periodically coordinates dirty page cache buffer writebacks and filesystem synchronization.

Kernel threads are automatically distinguished in supervisor diagnostics (``pidinfo``) and userland status tools (``ps``) using bracket notation (e.g. ``[kthreadd]``) and returning zero-length command line buffers in ``/proc/<pid>/cmdline``.

3. In-Kernel Module Management & Supervisor Commands
====================================================

The module subsystem in ``crates/core/src/module/`` and ``crates/shell/src/cmds/dev.rs`` provides interactive module control in Ring 0:

* **``lsmod``**: Queries ``get_modules_snapshot()`` and renders an aligned tabular view of module name, footprint size, active reference count, lifecycle state and load address.
* **``insmod <name> [size]``**: Invokes ``register_module()`` to dynamically insert a new driver into ``MODULE_TABLE``.
* **``rmmod <name>``**: Invokes ``unregister_module()``, verifying that the target module has zero active references before transitioning it to unloaded state.
* **``modinfo <name>``**: Inspects module metadata including file name, author, license, size, reference count and virtual address.

4. ProcFS Telemetry (``/proc/modules`` & ``/proc/kallsyms``)
============================================================

Two canonical Linux introspection files were added to ``crates/fs/src/proc/system/``:

* **``/proc/modules``**: Formats the active module table into the Linux-standard space-delimited representation (``<name> <size> <refcount> - <state> <address>``).
* **``/proc/kallsyms``**: Formats the exported kernel symbol table (``BASE_KALLSYMS``) into address, type and symbol name triples (e.g. ``ffff800000101000 T vga_print_str``).

5. Reference Userland Core Utilities
====================================

To complement the supervisor commands, optional reference userspace binaries were authored in ``userland/coreutils/`` and ``userland/tests/``:

* **``/bin/lsmod.elf``**: Reads and renders ``/proc/modules``.
* **``/bin/insmod.elf``**: Dispatches module insertion requests via system call vector 34 (``sys_init_module``).
* **``/bin/rmmod.elf``**: Dispatches module removal requests via system call vector 35 (``sys_delete_module``).
* **``/bin/test_modules.elf``**: Automated conformance test suite verifying ProcFS nodes, dynamic registration and teardown.

6. Dual-Architecture Verification
=================================

All components were validated symmetrically across ``x86_64`` and ``i686`` bare-metal environments:

* 100% compliance with Keira coding standards (zero Oxford commas, single space between sentences and SPDX headers).
* Full multi-architecture automated smoke tests passing without warnings or errors.
* Live QEMU verification capturing supervisor command execution and process table telemetry.
