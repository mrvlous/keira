.. SPDX-License-Identifier: GPL-2.0-only


Developer Quality Gate & Verification Checklist
===============================================

This document outlines the required pre-submission validation pipeline, test execution targets and code formatting checks that every contribution must pass before review.

Local Validation Sequence
=========================

Before opening a pull request or submitting patches, execute the full validation sequence locally:

1. Toolchain & Environment Verification
---------------------------------------

Verify that all required toolchain dependencies and linters are installed:

.. code-block:: bash

   # Audits 15 system toolchain dependencies with installation commands on missing tools
   make check

2. Code Formatting & EditorConfig Compliance
--------------------------------------------

Verify that all Rust crates, userland C and Python scripts adhere to formatting specifications:

.. code-block:: bash

   # Formats Rust crates with cargo fmt and userland C with clang-format
   make format

3. Static Analysis & Linting
----------------------------

Perform static analysis on userland C code, Sphinx documentation and kernel scripts:

.. code-block:: bash

   # Executes clang-tidy static analysis on userland C
   make lint

   # Validates patch style, trailing whitespace, SPDX identifiers and coding rules
   python3 scripts/checkpatch.py <changed files>

4. Documentation Style & SPDX Verification
------------------------------------------

Verify that all reStructuredText files adhere to style guide rules:

.. code-block:: bash

   # Validates SPDX headers, character counts and zero Oxford comma rules
   make docs-lint

5. Host Unit Testing
--------------------

Execute all host unit tests across the 12 kernel crates:

.. code-block:: bash

   # Runs 246 cargo tests and doc-tests
   make test-unit

6. Userland Diagnostic Testing
------------------------------

Compile and verify Ring 3 userland ABI test executables:

.. code-block:: bash

   make user-test ARCH=x86_64
   make user-test ARCH=i686

7. Multi-Architecture Build & ISO Generation
--------------------------------------------

Compile kernel binaries, userland disk images and bootable ISOs for both target architectures:

.. code-block:: bash

   make full

8. Automated Headless Smoke Boot Tests
--------------------------------------

Run QEMU automated boot verification across both architectures:

.. code-block:: bash

   make test-all

Pre-Flight Checklist
====================

Ensure all the following criteria are fulfilled before submitting:

* [ ] Dual-architecture parity: builds and boots on both ``x86_64`` and ``i686``.
* [ ] Zero warnings policy: zero compiler warnings or lint errors.
* [ ] Validated with ``scripts/checkpatch.py`` and ``.pylintrc`` for Python scripts.
* [ ] Strict ``.editorconfig`` compliance (8-column tabs for Kconfig and Kbuild, 4 spaces for Rust, C and Python).
* [ ] Valid SPDX-License-Identifier headers across all source, build and configuration files.
* [ ] Zero Oxford commas across all commit messages and documentation prose.
* [ ] Zero double whitespaces in prose and configuration files.
* [ ] Every commit is atomic, bisectable and has a ``Signed-off-by`` trailer.
