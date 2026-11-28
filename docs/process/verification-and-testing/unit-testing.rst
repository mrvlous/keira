.. SPDX-License-Identifier: GPL-2.0-only


Host-Side Unit Testing Architecture
===================================

Keira verifies pure kernel logic, configuration tooling and selftest harnesses on the host prior to bare-metal cross-compilation and QEMU execution.

1. Unified Host Unit Test Target
================================

Execute all host-side Python and Rust unit test suites through a single entrypoint:

.. code-block:: bash

   # Run Kconfig tests, QEMU smoke selftest unit tests and Rust workspace tests
   make test-unit

2. Python Tooling & Selftest Suites
===================================

1. **Kconfig & Kbuild Engine (``scripts/kconfig/tests/runner.py``)**:

   .. code-block:: bash

      make test-kbuild

   Validates grammar parsing, symbol visibility, dependency evaluation and defconfig header generation without external Python dependencies.

2. **QEMU Smoke-Test Selftest Runner (``scripts/selftests/tests/runner.py``)**:

   .. code-block:: bash

      make test-smoke

   Verifies serial boot marker validation, kernel panic and reboot-loop detection, non-zero exit handling and post-boot grace-period scheduling (``TestEvaluateResult`` and ``TestRunQemu``).

3. Rust Workspace Crate Tests
=============================

Runs all crate unit tests sequentially (``cargo test --workspace -- --test-threads=1``) across ``keira-arch``, ``keira-core``, ``keira-crypto``, ``keira-fs``, ``keira-io``, ``keira-ipc``, ``keira-kernel``, ``keira-mem``, ``keira-net``, ``keira-shell``, ``keira-syscall`` and ``keira-task``.
