.. SPDX-License-Identifier: GPL-2.0-only


Kernel Command-Line Boot Parameters
===================================

Keira accepts kernel boot parameters supplied through the Multiboot2 command-line tag (up to 256 bytes in ``crates/core/src/cmdline.rs``). If no explicit bootloader command line is supplied, the kernel defaults to ``console=tty0 quiet`` when userland is compiled in or ``console=ring0 quiet`` when ``CONFIG_USERLAND=n``.

1. Supported Boot Parameters
============================

.. list-table::
   :header-rows: 1
   :widths: 22 20 58

   * - Parameter
     - Default Value
     - Kernel Behavior and Parsing Location
   * - ``console=tty0``
     - Active when ``CONFIG_USERLAND=y``
     - Allows automatic transition from Ring 0 bootstrap into Ring 3 userspace (``init`` followed by ``/bin/sh``) in ``crates/kernel/src/runtime/main_loop.rs``.
   * - ``console=ring0``
     - Active when ``CONFIG_USERLAND=n``
     - Skips userspace ``init`` and ``sh`` startup even if binaries are attached, landing directly into the Ring 0 supervisor console (``ring0@keira:/#``).
   * - ``init=<path>``
     - ``/bin/init.elf``
     - Overrides the default early userspace PID 1 binary path checked via ``keira_fs::exists()`` before falling back to ``/bin/init.elf`` and ``/bin/init``.
   * - ``root=<device>``
     - ``/dev/sda``
     - Identifies the root block device passed from GRUB ``grub.cfg`` when attaching the FAT16 root filesystem disk image.
   * - ``quiet``
     - Enabled
     - Suppresses non-critical verbose startup chatter on early console output.

2. Bootloader Configuration Examples
====================================

Standard Ring 3 Userspace Entry (``USERLAND=1``):

.. code-block:: text

   menuentry "Keira Kernel" {
       multiboot2 /boot/keira.bin console=tty0 root=/dev/sda quiet
       module2 /boot/initrd.tar initrd
       boot
   }

Direct Ring 0 Supervisor Rescue Entry (``USERLAND=0`` or Override):

.. code-block:: text

   menuentry "Keira Kernel (Ring 0)" {
       multiboot2 /boot/keira.bin console=ring0 quiet
       boot
   }

3. Programmatic Parsing API (``keira-core``)
============================================

Subsystem code inspects bootloader arguments without heap allocation via ``keira_core::cmdline``:

- ``keira_core::has_boot_cmdline_flag(flag)``: Checks whether an exact whitespace-delimited token (such as ``console=ring0``) is present.
- ``keira_core::get_boot_cmdline_value(key, out)``: Extracts ``key=<value>`` pairs (such as ``init=/bin/sh.elf``) into a caller-supplied stack buffer.
- ``keira_core::copy_boot_cmdline(out)``: Copies the active boot command line for diagnostic output in supervisor commands and tests.
