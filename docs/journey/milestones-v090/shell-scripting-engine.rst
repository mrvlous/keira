.. SPDX-License-Identifier: GPL-2.0-only

Milestone 61: POSIX Shell Scripting Language Engine
===================================================

Milestone 61 transforms Keira's freestanding Ring 3 userspace shell (``sh.elf``)
into a structured, programmable POSIX scripting language engine. Building upon
the command execution, I/O redirection and job control primitives established
in earlier milestones, Milestone 61 enables dynamic shell automation, variable
interpolation, conditional branching and iterative loops directly on bare-metal hardware.

Historical Context & The Static Scripting Barrier
-------------------------------------------------

Prior to Milestone 61, Keira's userspace shell supported interactive command dispatch,
pipes (``|``), file redirection (``<``, ``>``, ``>>``) and shebang script execution
(``#!/bin/sh``). However, its script execution model was strictly sequential and line-oriented.
It lacked core language capabilities required for modern system administration:

1. **Absence of Variable State**: Shell variables could only be viewed via environment
   queries, without support for local variable stores or in-place parameter interpolation.
2. **Lack of Conditional Branching**: Scripts could not inspect runtime conditions or
   branch dynamically via ``if-then-elif-else-fi`` blocks.
3. **No Iterative Constructs**: Repeating tasks required unrolled commands, as the shell
   lacked ``for`` and ``while`` loop support.
4. **Missing Condition Evaluator**: The system lacked standard POSIX ``test`` and ``[``
   expression evaluation tools for inspecting file attributes and string values.

Architectural Design & Structured Language Engine
-------------------------------------------------

Milestone 61 addresses these challenges by introducing a lightweight, freestanding
POSIX scripting engine inside ``userland/shell/sh.c``:

* **Variable Symbol Table & Parameter Expansion**: Internal key-value symbol table
  storing active local and exported shell variables. Parameter expander supporting
  ``$VAR``, ``${VAR}``, ``$?`` (last exit code), ``$$`` (PID), ``$#`` (positional
  parameter count) and ``$0`` .. ``$9`` (script arguments). Single quotes (``'...'``)
  strictly preserve literal contents, while double quotes (``"..."``) permit variable
  interpolation.
* **Standalone & Builtin Condition Evaluator (test / [)**: Implements ``test`` and
  ``[`` expression evaluation natively as a shell builtin and exports it as standalone
  core utility binaries (``/bin/test`` and ``/bin/[``). Supports file existence (``-e``),
  file types (``-f``, ``-d``), non-zero size (``-s``), access permissions (``-r``,
  ``-w``, ``-x``), string equality (``=``, ``!=``), string length (``-z``, ``-n``),
  integer arithmetic (``-eq``, ``-ne``, ``-lt``, ``-le``, ``-gt``, ``-ge``) and
  logical grouping (``!``, ``-a``, ``-o``).
* **Compound Control Flow & Block Parsing**: Multi-branch conditionals (``if CMD; then
  CMDS; elif CMD; then CMDS; else CMDS; fi``), iterative for loops (``for VAR in WORDS;
  do CMDS; done``), condition-driven while loops (``while CMD; do CMDS; done``) and loop
  control builtins (``break`` and ``continue``). Script sourcing (``source <file>`` and
  ``. <file>``) executing in the parent shell context. Multiline block parser buffering
  compound statements across interactive turns with secondary prompt (``>``).

Dual-Architecture Verification
------------------------------

Milestone 61 maintains strict dual-architecture parity across modern 64-bit ``x86_64``
and legacy 32-bit ``i686`` targets. Conformance is verified through dedicated automated
runners:

* Canonical automated test script: ``/etc/test_scripting.sh``
* Standalone C test runner: ``/bin/test_sh_scripting.elf``
* Automated QEMU headless validation across both target architectures
