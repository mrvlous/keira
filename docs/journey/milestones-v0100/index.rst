.. SPDX-License-Identifier: GPL-2.0-only


Milestones v0.10.0: Job Control, Demarcation & Kconfig/Kbuild Architecture
==========================================================================

Chronicles for Milestones 57-70: Process group isolation, session management, shell job control, pure kernel demarcation, Linux-grade Kconfig architecture, modular Kbuild system, POSIX shell scripting language engine, pure kernel infrastructure, in-kernel ELF relocatable module loader, active background kernel daemons ([kswapd0] & [kflushd]), developer introspection tooling (bloat-o-meter & faddr2line), binary buddy allocator with physical memory zones, kernel wait queues with sleep synchronization, kernel workqueues with [kworker/0:0], end-to-end TLS 1.3 HTTPS networking and full Read-Write EXT4 with [kflushd] page cache writeback.

Subtopics & Specifications
==========================

.. toctree::
   :maxdepth: 1

   job-control
   pure-kernel-demarcation
   kconfig-integration
   kbuild-integration
   shell-scripting-engine
   pure-kernel-infrastructure
   elf-module-loader
   active-kernel-daemons
   kernel-tooling-architecture
   buddy-allocator-and-zones
   kernel-waitqueues
   kernel-workqueues
   tls-https-transport
   ext4-rw-writeback
