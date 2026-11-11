.. SPDX-License-Identifier: GPL-2.0-only

============================================================================================================
Milestone 50: Virtual Memory Areas (VMA) Address Space Inspection, /proc/[pid]/maps & Userspace pmap Utility
============================================================================================================

1. Executive Summary
====================

Milestone 50 establishes complete virtual memory area (VMA) address space inspection and introspection within Keira Kernel. Prior to this milestone, dynamically allocated memory areas (``mmap``/``munmap``) and process segments (text, data, BSS, heap and stack) were tracked in internal kernel tables but remained opaque to userspace and supervisor diagnostics. When a process performed ``fork()``, active VMA records were not inherited by the child PML4 root, preventing dynamic tracking of cloned anonymous mappings. Furthermore, userspace lacked a standard ``/proc/[pid]/maps`` virtual filesystem interface and a canonical ``pmap`` utility.

Through Milestone 50, Keira introduces:

1. **VMM VMA Operations Hardening**: Implementation of ``get_vmas_for_pml4``, ``clone_vmas_for_pml4`` and ``total_active_vmas`` in ``crates/mem``, ensuring robust querying and seamless VMA inheritance across both standard and Copy-on-Write (CoW) fork paths.
2. **Canonical Linux ``/proc/[pid]/maps`` Synthesis**: Implementation of ``read_task_maps`` in ``crates/fs`` and dynamic memory region synthesis in ``crates/task``, providing complete 6-column Linux-standard map records (``<start>-<end> <perms> <offset> <dev> <inode> <pathname>``) for text, rodata, data/bss, heap, dynamic VMAs and user/kernel stacks.
3. **Freestanding Userspace ``pmap.elf`` Utility**: Implementation of ``/bin/pmap.elf`` and ``/bin/pmap`` in freestanding C, expanding Keira's core utility suite to 42 standalone binaries.
4. **Supervisor ``vmainfo`` Diagnostic Command**: Implementation of the ``vmainfo`` (alias: ``vma``) command in ``crates/shell``, offering per-PID and system-wide (``-a``) virtual memory layout inspection.
5. **ABI Compliance Test Suite Expansion**: Test 37 in ``/bin/test_abi.elf`` verifying dynamic reflection of ``mmap`` allocations and ``munmap`` reclamation within ``/proc/self/maps``.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph MemoryLayer["Virtual Memory Management (crates/mem)"]
           VMA_TBL["VMA_TABLE (MAX_VMAS = 128)"]
           CLONE["clone_vmas_for_pml4(parent, child)"]
           QUERY["get_vmas_for_pml4(pml4, out_vmas)"]
           VMA_TBL --> QUERY
           CLONE --> VMA_TBL
       end

       subgraph TaskLayer["Process Scheduler & Address Space (crates/task)"]
           TASK["Task Descriptor (Text, Data, Heap, Stack)"]
           HOOK["TaskMapsProvider Hook"]
           TASK --> HOOK
           QUERY --> HOOK
       end

       subgraph FsLayer["ProcFS Virtual Filesystem (crates/fs)"]
           DISPATCH["ProcFS Dispatcher (ino 8: 'maps')"]
           HOOK --> DISPATCH
           DISPATCH --> PROC_MAPS["/proc/[pid]/maps & /proc/self/maps"]
       end

       subgraph Interfaces["Observability & Diagnostics"]
           PROC_MAPS --> PMAP["Userland /bin/pmap.elf (42nd Utility)"]
           PROC_MAPS --> CAT["Userland /bin/cat.elf /proc/self/maps"]
           VMA_TBL --> VMAINFO["Supervisor 'vmainfo [pid]' & 'vmainfo -a'"]
       end

2.1 VMA Table Operations & Fork Inheritance
-------------------------------------------

Dynamic virtual memory allocations made via ``sys_mmap`` (vector 9) are recorded in ``VMA_TABLE`` located in ``crates/mem/src/vmm/area/table.rs``. Milestone 50 introduces query and inheritance routines:

* **``get_vmas_for_pml4(pml4: u64, out: &mut [VirtualMemoryArea]) -> usize``**: Scans the VMA table under spinlock protection and copies active VMAs belonging to the specified page directory into the caller-provided buffer.
* **``clone_vmas_for_pml4(parent_pml4: u64, child_pml4: u64)``**: Called during ``clone_user_address_space`` and ``clone_user_address_space_cow``. Duplicates all active VMAs from the parent to the child address space, ensuring dynamic mappings survive process branching.
* **``total_active_vmas() -> usize``**: Returns the global count of active VMA slots across all active address spaces.

2.2 Canonical Linux /proc/[pid]/maps Formatting
-----------------------------------------------

The synthesis engine in ``crates/task/src/scheduler/core/hooks.rs`` produces canonical Linux format lines:

.. math::

   \text{address range} \quad \text{perms} \quad \text{offset} \quad \text{device} \quad \text{inode} \quad \text{pathname}

* **Address Range**: Fixed 8-digit hexadecimal width for 32-bit addresses (``00400000-00405000``) and 12-digit width for 64-bit addresses (``7fffffdde000-7fffffdfe000``).
* **Permissions**: Standard 4-character POSIX flag format: ``r`` (read), ``w`` (write), ``x`` (execute) and ``p``/``s`` (private or shared).
* **Offset**: File offset in 8-digit hexadecimal format (``00000000``).
* **Device & Inode**: Block device major/minor pair (``00:00``) and VFS inode number (``0``).
* **Pathname**: Descriptive label or executable file path (``/bin/init.elf``, ``[heap]``, ``[stack]``, ``[kernel_stack]``).

2.3 Freestanding pmap.elf Core Utility
--------------------------------------

The ``pmap`` utility (``userland/bin/pmap/main.c``) is implemented in freestanding C without external dependencies:

* Opens and parses ``/proc/[pid]/maps`` using raw system calls.
* Parses hexadecimal start and end boundaries to compute the exact mapping size in KiB.
* Prints formatted columnar output with total address space calculation.
* Supports standard flags: ``-x`` (extended format), ``-d`` (device format), ``-q`` (quiet mode) and ``-h`` (help).

3. Observability & Userland Verification
========================================

3.1 Inspecting Address Space with pmap.elf
------------------------------------------

.. code-block:: text

   keira:/# pmap.elf 1
   1: /bin/init.elf
   0000000000400000      16K r-xp 00000000 00:00 0 /bin/init.elf
   0000000000404000       4K r--p 00004000 00:00 0 /bin/init.elf
   0000000000405000       4K rw-p 00005000 00:00 0 /bin/init.elf
   0000000000406000      64K rw-p 00000000 00:00 0 [heap]
   00007fffffdde000     128K rw-p 00000000 00:00 0 [stack]
    total               216K

3.2 Reading /proc/self/maps Directly
------------------------------------

.. code-block:: text

   keira:/# cat.elf /proc/1/maps
   00400000-00404000 r-xp 00000000 00:00 0 /bin/init.elf
   00404000-00405000 r--p 00004000 00:00 0 /bin/init.elf
   00405000-00406000 rw-p 00005000 00:00 0 /bin/init.elf
   00406000-00416000 rw-p 00000000 00:00 0 [heap]
   7fffffdde000-7fffffdfe000 rw-p 00000000 00:00 0 [stack]

3.3 Supervisor Diagnostic Command (vmainfo)
-------------------------------------------

.. code-block:: text

   ring0@keira:/# vmainfo 0
   Virtual Memory Areas (VMAs) for Task [kernel_shell] (PID 0):
     Start Addr   End Addr     Size     Perms Share Backing Path / Name
     ----------------------------------------------------------------------
     0x00100000 - 0x00400000   3072 KB  rwxp  priv  [kernel_image]
     0x07FFE000 - 0x08000000   8 KB     rw-p  priv  [kernel_stack]

     PML4 Base    : 0x00200000
     Active VMAs  : 2 regions
     VMA Capacity : 0 / 128 slots in kernel

4. Automated Testing & Verification
===================================

Milestone 50 passed complete verification:

1. **Unit Tests**: All 12 modular kernel crates passed unit testing with 0 errors (``cargo test --workspace``).
2. **ABI Compliance**: Added Test 37 to ``/bin/test_abi.elf``. The test opens ``/proc/self/maps``, verifies ``[stack]`` presence, allocates a 16 KiB anonymous mapping via ``mmap``, asserts its immediate reflection in ``/proc/self/maps`` and verifies its clean unmapping via ``munmap``. All 37 ABI compliance tests passed.
3. **Dual-Architecture Toolchain**: Built and verified clean compilation for both ``x86_64`` and ``i686`` targets (``make full``).
4. **Runtime Userland Execution**: Successfully executed native KCC compiler building userland binaries, downloaded resources via network ``fetch.elf`` and inspected live VMAs across processes.
