.. SPDX-License-Identifier: GPL-2.0-only


Milestone 58: Pure Kernel Demarcation, Distro Bloat Eradication & Dual-Ring Help Architecture
=============================================================================================

Milestone 58 advances Keira's architectural clarity as a pure freestanding monolithic kernel in Ring 0. This milestone purges residual distribution-level bloat (specifically the artificial ``/etc/os-release`` file), refactors the Ring 0 supervisor console ``help`` command into a streamlined single-screen categorized matrix, implements native built-in ``help`` within the Ring 3 interactive shell (``sh.elf``) and formalizes the privilege boundary separating the autonomous supervisor from the optional reference userspace.

1. Architectural Motivation & Pure Kernel Purity
================================================

Keira is fundamentally a bare-metal monolithic kernel written from first principles, not an operating system distribution. Prior to Milestone 58:

1. **Residual Distro Artifacts**: The root filesystem staging script generated an artificial ``/etc/os-release`` file conforming to freedesktop specifications. This created an ambiguous impression that Keira was an OS distribution rather than a pure freestanding kernel.
2. **Ring 0 Supervisor Verbosity**: The supervisor console's ``help`` command dumped 35 lines of verbose descriptions and hardcoded userland package assumptions (``Core Utilities (/bin): 42 native ELF binaries``). On standard 80x25 VGA text displays, this overflowed the screen and scrolled the first 10 lines off-screen.
3. **Ring 3 Built-in Discovery**: In the Ring 3 userspace shell (``sh.elf``), typing ``help`` printed ``command not found: help``. Users lacked an immediate in-shell method to discover shell built-ins such as ``jobs``, ``fg``, ``bg``, ``history`` and ``exit``.

Milestone 58 resolves these issues comprehensively while preserving full dual-architecture parity across ``x86_64`` and ``i686``.

2. Distro Bloat Eradication & Canonical Configuration
=====================================================

The build system staging rules in ``Makefile`` were audited and cleaned:

- **Eradication of ``/etc/os-release``**: The generation of ``/etc/os-release`` was permanently removed from both minimal (``USERLAND=0``) and canonical (``USERLAND=1``) staging recipes.
- **Sole Source of Truth (``/etc/kernel.cfg``)**: The kernel relies strictly on ``/etc/kernel.cfg`` for build-time metadata (kernel name, target version, architecture and userland enablement flag) alongside canonical POSIX network configuration files (``/etc/hostname`` and ``/etc/hosts``).
- **Zero Ring 0 Distro Assumptions**: All hardcoded assumptions about external userland binary counts were removed from the kernel supervisor crate.

3. Streamlined Ring 0 Supervisor Help Matrix
============================================

In ``crates/shell/src/cmds/util.rs``, the ``help`` command was restructured into an ergonomic, categorized 9-line matrix that guarantees zero screen scrolling on 80x25 VGA consoles:

.. code-block:: text

   Keira Kernel Diagnostic Console (Ring 0 v0.9.0)

   Diagnostics : system   cpu      smp      memory   slabinfo  regs     irq
   Hardware    : devices  drivers  disk     dcache   icache    pipes
   Security    : secinfo  aslrinfo rnginfo  rlimit   unixsock  vmainfo
   Multiplex   : pollinfo selectinfo
   Control     : syslog   wipe     sync     unwind   sh        run <path>
   Lifecycle   : reset    power

   Type '<cmd> --help' for options. Press Tab for completion.

Subsystem commands are organized logically across Diagnostics, Hardware, Security, Multiplex, Control and Lifecycle. Detailed flag options are delegated to individual command help (``<cmd> --help``), and interactive discovery is supported via hardware Tab auto-completion.

4. Ring 3 Shell Native Built-in Help
====================================

In ``userland/shell/sh.c``, a native ``builtin_help()`` routine was implemented to handle the ``help`` command directly inside the interactive command interpreter:

.. code-block:: text

   Keira Userspace Shell Console (Ring 3 v0.9.0)

   Navigation  : cd [dir]   pwd
   Environment : export [var=val]
   Job Control : jobs       fg [id]  bg [id]
   Session     : history    clear    help     exit
   Utilities   : cat, ls, ps, df, fetch, kcc (Type 'ls /bin' for all binaries)

   Type '<binary> --help' for options. Type 'exit' to return to Ring 0.

This built-in documentation clarifies the job control features introduced in Milestone 57, explains the privilege transition back to Ring 0 via ``exit`` and instructs users on navigating available system binaries.

5. Architectural Demarcation & Privilege Isolation
==================================================

A comprehensive architectural manual was authored in ``docs/arch/ring-demarcation.rst`` formalizing:

1. **Hardware Privilege Barriers**: GDT segment descriptors (``0x08``, ``0x10``, ``0x1B`` and ``0x23``), TSS stack pointers (``rsp0``/``esp0``) and fast system call MSRs (``STAR``, ``LSTAR``, ``FMASK``).
2. **Virtual Memory Isolation**: Higher-half kernel mapping (``0xFFFF_8000_0000_0000`` / ``0xC000_0000``), W^X execution protection and silicon-backed ASLR entropy.
3. **Safe Memory Validation**: Bounded ``copy_from_user`` and ``copy_to_user`` primitives preventing kernel memory dereferencing vulnerabilities.
4. **Fault Containment**: Strict confinement of userspace exceptions (``SIGSEGV``, ``SIGILL``) to prevent Ring 0 supervisor instability.
5. **Clean Handover**: Graceful return from userspace exit to the Ring 0 supervisor console prompt (``ring0@keira:/#``).
