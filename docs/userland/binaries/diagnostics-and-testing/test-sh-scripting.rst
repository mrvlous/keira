.. SPDX-License-Identifier: GPL-2.0-only


test_sh_scripting.elf Shell Scripting Conformance Suite
=======================================================

The ``test_sh_scripting.elf`` executable (``/bin/test_sh_scripting.elf``) is a dedicated Ring 3 verification binary that executes comprehensive conformance tests against Keira's POSIX userspace shell scripting engine.

1. Architectural Purpose
========================

Introduced in Milestone 61, this test suite verifies that ``sh.elf`` correctly evaluates structured scripting constructs across dual-architecture targets:

1. **Parameter Expansion**: Asserts variable assignment, interpolation, positional parameter bindings (``$0``, ``$1``, ``$#``) and special status parameters (``$?``, ``$$``).
2. **Conditional Execution**: Tests multi-branch ``if-then-elif-else-fi`` statements and verifies correct branch selection based on process exit statuses.
3. **Iterative Loops**: Verifies ``for-in-do-done`` word iterations and ``while-do-done`` condition loops with counter updates.
4. **Canonical Script Dispatch**: Spawns and executes ``/etc/test_scripting.sh`` on 64-bit targets via ``execve()`` and ``waitpid()``, asserting exit status propagation and shebang dispatch. On single-task 32-bit targets (``i686``), subprocess spawning is skipped and ``sh.elf /etc/test_scripting.sh`` validates direct script execution.

2. Test Execution & Output
==========================

.. code-block:: bash

   keira:/# test_sh_scripting.elf
   ======================================================================
   Keira Kernel - Shell Scripting Engine Conformance Suite
   ======================================================================
   [TEST 1] Testing variable assignment and interpolation...
     [PASS] Variable assignment and expansion verified
   [TEST 2] Testing conditional branching (if-then-elif-else-fi)...
     [PASS] Multi-branch conditional logic verified
   [TEST 3] Testing iterative for loop (for-in-do-done)...
     [PASS] For loop word iteration verified
   [TEST 4] Testing iterative while loop (while-do-done)...
     [PASS] While loop execution verified
   [TEST 5] Executing canonical script file /etc/test_scripting.sh...
     [PASS] Script file execution completed with status 0
   ======================================================================
   [SUCCESS] All 5 shell scripting engine tests passed cleanly!
