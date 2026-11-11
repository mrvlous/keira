.. SPDX-License-Identifier: GPL-2.0-only

====================================
Keira Kernel Technical Documentation
====================================

Welcome to the official technical architecture and reference documentation for the **Keira Kernel**.

System Overview
===============

Keira is an educational, exploratory freestanding monolithic kernel operating in Ring 0,
built from scratch in safe Rust, Assembly and C. It features dual bare-metal architecture support
for modern 64-bit Long Mode (``x86_64``) and pure 32-bit Protected Mode (``i686``).

The documentation suite is organized into 16 specialized domains modeling the Linux Kernel
documentation architecture:

.. toctree::
   :maxdepth: 2
   :caption: System Administration & Operations

   admin-guide/index

.. toctree::
   :maxdepth: 2
   :caption: Hardware Architecture & Bootstrap

   arch/index

.. toctree::
   :maxdepth: 2
   :caption: Core Infrastructure & Primitives

   core-api/index

.. toctree::
   :maxdepth: 2
   :caption: Memory Management Subsystem

   memory-management/index

.. toctree::
   :maxdepth: 2
   :caption: Process Scheduler & Concurrency

   scheduler/index

.. toctree::
   :maxdepth: 2
   :caption: Hardware Device Drivers

   driver-api/index

.. toctree::
   :maxdepth: 2
   :caption: Filesystems & Storage Layer

   filesystems/index

.. toctree::
   :maxdepth: 2
   :caption: Networking & BSD Sockets

   networking/index

.. toctree::
   :maxdepth: 2
   :caption: Inter-Process Communication (IPC)

   ipc/index

.. toctree::
   :maxdepth: 2
   :caption: Security, Hardening & Enclaves

   security/index

.. toctree::
   :maxdepth: 2
   :caption: Userspace API & System Calls

   userspace-api/index

.. toctree::
   :maxdepth: 2
   :caption: Ring 0 Supervisor Console

   supervisor-shell/index

.. toctree::
   :maxdepth: 2
   :caption: Userland C SDK & Utilities

   userland/index

.. toctree::
   :maxdepth: 2
   :caption: Development Process & Style

   process/index

.. toctree::
   :maxdepth: 2
   :caption: Architectural Learning Journey

   journey/index

.. toctree::
   :maxdepth: 2
   :caption: Bare-Metal Target Profiles

   targets/index

Indices and Tables
==================

* :ref:`genindex`
* :ref:`search`
