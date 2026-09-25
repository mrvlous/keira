<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Fault Containment & Core Dumps

When an unprivileged process triggers an exception (`#DE`, `#UD`, `#PF`):
1. Safely contains the fault without halting the kernel.
2. Writes a diagnostic core dump to `/data/log/core_<pid>.dmp`.
3. Converts the fault into the appropriate POSIX signal (`SIGFPE`, `SIGILL`, `SIGSEGV`).
