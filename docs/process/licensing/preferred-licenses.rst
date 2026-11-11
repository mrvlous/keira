.. SPDX-License-Identifier: GPL-2.0-only

=================================
Preferred Licenses (GPL-2.0-only)
=================================

The Keira Kernel is free and open-source software distributed under the terms of the GNU General Public License version 2 only (GPL-2.0-only), matching the licensing philosophy of the upstream Linux Kernel.

1. Primary Kernel License
=========================

The entire Ring 0 kernel tree, architecture bootstrap routines, device drivers and kernel libraries are licensed under:

.. code-block:: text

   GPL-2.0-only

Contributions submitted to the kernel core must be licensed under ``GPL-2.0-only``. Submissions under ``GPL-2.0-or-later`` or ``GPL-3.0`` are not accepted for core kernel components to prevent license fragmentation.

2. Permissible Compatible Licenses
==================================

Certain sub-components, standalone crates and userland libraries may employ permissive licenses to facilitate broad adoption:

.. list-table:: Compatible Subsystem Licenses
   :widths: 25 35 40
   :header-rows: 1

   * - License
     - SPDX Identifier
     - Permissible Location
   * - GNU GPL v2.0 only
     - ``GPL-2.0-only``
     - Ring 0 Kernel, Drivers, Subsystems & Architecture code
   * - MIT License
     - ``MIT``
     - Independent userland utility crates & standalone tools
   * - BSD 2-Clause
     - ``BSD-2-Clause``
     - Architecture header definitions and UAPI exports
   * - Apache 2.0
     - ``Apache-2.0``
     - Developer tooling and build system scripts

3. Dual Licensing
=================

Where components are intended for use outside the kernel tree (such as shared cryptographic algorithms or serialization formats), dual licensing under ``GPL-2.0-only OR MIT`` is permissible with explicit maintainer approval.
