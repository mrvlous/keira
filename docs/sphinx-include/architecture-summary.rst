.. SPDX-License-Identifier: GPL-2.0-only

.. list-table:: Target Architecture Specifications
   :widths: 25 35 40
   :header-rows: 1

   * - Architecture Target
     - Privilege Model
     - Paging & Memory Bounds
   * - ``x86_64-unknown-none``
     - Ring 0 Kernel / Ring 3 Userland
     - 4-Level Paging (PML4), Higher-Half ``0xFFFF800000000000``
   * - ``i686-unknown-none``
     - Ring 0 Kernel / Ring 3 Userland
     - 2-Level Paging, Higher-Half ``0xC0000000``
