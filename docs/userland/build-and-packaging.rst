.. SPDX-License-Identifier: GPL-2.0-only


Keira Userland Runtime, Native C Compiler & POSIX C Library
===========================================================

The ``userland`` domain comprises the Ring 3 execution environment, architecture bootstrap stubs, standard C library (libc), native C compiler (``kcc``), Ring 3 userspace binaries and multi-user system configuration.

Userland Subsystem Architecture
===============================

.. code-block:: text

   graph TD
       Kernel["Keira Microkernel / Syscall Dispatcher"] <--> Trap["INT 0x80 / SYSCALL / SYSENTER"]
       Trap <--> Libc["libc/<br/>POSIX Standard C Library & Syscall Wrappers"]
       Arch["arch/<br/>i686 & x86_64 crt0.S & Linker Scripts"] --> Binaries["binaries/<br/>Userland Executable Binaries"]
       Libc --> Binaries
       Compiler["compiler/<br/>KCC Native In-Kernel C Compiler"] --> Binaries
       System["system/<br/>Credentials, Permissions, Hostname & Posix I/O"] --> Binaries

Submodule Index
===============

.. list-table::
   :header-rows: 1
   :widths: 49 21 631

   * - Submodule
     - Focus Area
     - Description
   * - ``arch/``
     - Architecture Stubs
     - CRT0 entry stubs (``crt0.S``) and ELF linker scripts (``linker.ld``) for i686 and x86_64
   * - ``libc/``
     - Standard C Library
     - Hyper-modular ISO C and POSIX libc routines, headers and system call wrappers
   * - ``compiler/``
     - KCC Native Compiler
     - Preprocessor, recursive descent parser, lexer, AST and x86 code generator
   * - ``binaries/``
     - Ring 3 Binaries
     - 46 native ELF executables: ``init.elf``, ``sh.elf``, ``cat.elf``, ``chmod.elf``, ``chown.elf``, ``ls.elf``, ``grep.elf``, ``wc.elf``, ``head.elf``, ``tail.elf``, ``echo.elf``, ``sleep.elf``, ``fetch.elf``, ``kcc.elf``, ``ps.elf``, ``kill.elf``, ``hostname.elf``, ``clear.elf``, ``dmesg.elf``, ``df.elf``, ``ping.elf``, ``ifconfig.elf``, ``route.elf``, ``arp.elf``, ``netstat.elf``, ``iptables.elf``, ``uname.elf``, ``whoami.elf``, ``id.elf``, ``su.elf``, ``mkdir.elf``, ``touch.elf``, ``rm.elf``, ``ln.elf``, ``mount.elf``, ``umount.elf``, ``sysinfo.elf``, ``pmap.elf``, ``poweroff.elf``, ``reboot.elf``, ``test_pipe.elf``, ``test_threads.elf``, ``test_abi.elf``, ``fuzz_abi.elf``, ``test_poll.elf`` and ``test_select.elf``
   * - ``system/``
     - Userland Runtime
     - Multi-user credentials, file permissions, hostname configuration and initialization
   * - UAPI Boundary
     - Privilege Demarcation
     - Torvalds-grade Ring 0/Ring 3 privilege demarcation, UAPI headers and system call calling conventions
