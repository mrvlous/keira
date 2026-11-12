.. SPDX-License-Identifier: GPL-2.0-only


Linux Syscall Licensing Exception
=================================

The Keira Kernel provides standard POSIX and Linux-compatible system call interfaces to user space applications. To enable proprietary, permissive and copyleft user programs to run on Keira without legal encumbrance, the kernel adopts the standard Linux Syscall Exception.

1. Exception Scope & Grant
==========================

The Linux-syscall-note exception explicitly clarifies that user space software interacting with the kernel through standard system call boundaries does not constitute a derivative work of the kernel:

.. code-block:: text

   NOTE! This copyright does *not* cover user programs that use kernel
   services by normal system calls - this is merely considered normal use
   of the kernel and does *not* fall under the heading of "derived work".

2. Userspace API (UAPI) Headers
===============================

Header files located under ``userland/libc/include/sys/`` and kernel UAPI headers that define:

* System call vectors and vector numbers
* Structure layouts (e.g. ``stat``, ``termios``, ``dirent``)
* IOCTL command constants
* Error numbers and signal codes

are explicitly licensed with the ``Linux-syscall-note`` tag or dual-licensed to guarantee that linking against or including these headers does not obligate the calling user space application to be released under GPL-2.0-only.

3. Proprietary & Third-Party Binaries
=====================================

Developers may compile, execute and distribute proprietary or alternatively licensed user space applications on Keira without violating the kernel copyright or triggering copyleft redistribution clauses.
