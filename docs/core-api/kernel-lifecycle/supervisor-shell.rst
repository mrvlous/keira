.. SPDX-License-Identifier: GPL-2.0-only

===========================================
Keira Interactive Shell & Command Subsystem
===========================================

The ``shell`` domain provides the kernel emergency supervisor console control plane and 32 built-in diagnostic commands. Standard user utilities (``ps``, ``kill``, ``hostname``, ``dmesg``, ``df``, ``cat``, ``ls``, ``fetch``, ``kcc``, ``sh`` and ``init``) run exclusively in Ring 3 userspace (``/bin``).

Shell Architecture
==================

.. code-block:: text

   graph TD
       Shell["Interactive Shell"] --> Terminal["terminal/<br/>Prompt, Keyboard & Console Palette"]
       Shell --> Executor["executor/<br/>Command Dispatch, Autocomplete & CliArgs"]
       Shell --> Cmds["commands/<br/>32 Emergency Commands"]

Submodule Index
===============

.. list-table::
   :header-rows: 1
   :widths: 33 17 67

   * - Submodule
     - Focus Area
     - Description
   * - ``terminal/``
     - Terminal UI
     - Prompt rendering, keyboard input and monochrome palette
   * - ``executor/``
     - Command Execution
     - Argument parsing, pipe routing, autocomplete and execution dispatch
   * - ``commands/``
     - Built-in Commands
     - Reference manuals for all 32 built-in emergency commands
