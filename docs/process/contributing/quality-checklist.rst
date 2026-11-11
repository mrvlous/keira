.. SPDX-License-Identifier: GPL-2.0-only

===============================================
Developer Quality Gate & Verification Checklist
===============================================

This document outlines the required pre-submission validation pipeline, test execution targets and code formatting checks that every contribution must pass before review.

Local Validation Sequence
=========================

Before opening a pull request or submitting patches, execute the full validation sequence locally:

1. Code Formatting
------------------

Verify that all Rust and userland C source files follow consistent formatting rules:

.. code-block:: bash

   # Formats all Rust crates with cargo fmt and userland C with clang-format
   make format

2. Static Analysis & Linting
----------------------------

Perform static analysis on userland C code and Sphinx documentation:

.. code-block:: bash

   # Executes clang-tidy static analysis on userland C
   make lint

3. Documentation Style & SPDX Verification
------------------------------------------

Verify that all reStructuredText files adhere to style guide rules:

.. code-block:: bash

   # Validates SPDX headers, character counts and zero Oxford comma rules
   make docs-lint

4. Host Unit Testing
--------------------

Execute all host unit tests across the 12 kernel crates:

.. code-block:: bash

   # Runs 246 cargo tests and doc-tests
   make test-unit

5. Userland Diagnostic Testing
------------------------------

Compile and verify Ring 3 userland ABI test executables:

.. code-block:: bash

   make user-test ARCH=x86_64
   make user-test ARCH=i686

6. Multi-Architecture Build & ISO Generation
--------------------------------------------

Compile kernel binaries, userland disk images and bootable ISOs for both target architectures:

.. code-block:: bash

   make full

7. Automated Headless Smoke Boot Tests
--------------------------------------

Run QEMU automated boot verification across both architectures:

.. code-block:: bash

   make test-all

Pre-Flight Checklist
====================

Ensure all the following criteria are fulfilled before submitting:

* [ ] Dual-architecture parity: builds and boots on both ``x86_64`` and ``i686``.
* [ ] Zero warnings policy: zero compiler warnings or lint errors.
* [ ] Zero Oxford commas across all commit messages and documentation prose.
* [ ] Zero double whitespaces in prose and configuration files.
* [ ] Every commit is atomic, bisectable and has a ``Signed-off-by`` trailer.
