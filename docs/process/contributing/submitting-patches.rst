.. SPDX-License-Identifier: GPL-2.0-only


Submitting Patches & Pull Requests
==================================

This document describes the patch submission process, git commit hygiene, Signed-off-by requirements and pull request guidelines for Keira Kernel.

Commit Message Conventions
==========================

Keira follows the Conventional Commits specification tailored for systems kernel engineering:

.. code-block:: text

   <type>(<scope>): <subject>

   [optional body explaining technical motivation and architecture rationale]

   Signed-off-by: Author Name <author@example.com>

Commit Types
------------

* **feat**: A new kernel subsystem, driver capability, system call or userland utility.
* **fix**: A bug fix, race condition resolution, memory safety fix or fault containment correction.
* **refactor**: Code cleanup, architectural realignment or formatting without functional behavior change.
* **docs**: Documentation creation, technical correction or Sphinx documentation updates.
* **build**: Makefile improvements, toolchain configurations, linker scripts or CI automation.
* **test**: Adding or updating unit tests, integration tests, ABI fuzzing vectors or QEMU harnesses.
* **arch**: Architecture-specific low-level assembly, paging tables or GDT/IDT/TSS modifications.

Commit Scopes
-------------

The scope identifies the specific crate or subsystem modified:

* Kernel Subsystems: ``arch``, ``core``, ``crypto``, ``fs``, ``io``, ``ipc``, ``kernel``, ``mem``, ``net``, ``shell``, ``syscall``, ``task``
* Architecture: ``x86_64``, ``i686``, ``boot``, ``smp``
* Userland: ``userland``, ``libc``, ``kcc``, ``bin``
* Build & Process: ``makefile``, ``toolchain``, ``config``, ``process``, ``docs``

Atomic & Granular Commit Principles
===================================

Every commit must adhere to the atomic commit principle:

1. **Single Logical Change**: Each commit must encapsulate exactly one logical modification. Never bundle unrelated driver fixes, documentation changes and memory manager adjustments into a single commit.
2. **Bisectability**: Every single commit in a branch or pull request must build cleanly and pass automated unit tests. A commit that breaks the build or causes kernel panic will break ``git bisect``.
3. **No Decorative Comment Dividers**: Do not use decorative symbol dividers (such as repeated ``====`` or ``----``) in source code comments or assembly files. Reserve markdown/rst heading dividers exclusively for documentation.
4. **Style Guidelines Compliance**: Omit Oxford commas before "and" or "or" in commit messages, documentation and code comments. Eliminate all double whitespaces in prose and configuration files.

Signed-off-by & Developer Certificate of Origin
===============================================

All commits must include a ``Signed-off-by`` trailer affirming compliance with the Developer Certificate of Origin (DCO):

.. code-block:: text

   Signed-off-by: Moh. Ananda Firmansyah Putra <mrvlous@proton.me>

Pull Request Workflow
=====================

1. Create a focused topic branch from ``main`` (e.g., ``feat/virtio-net`` or ``fix/page-fault-unmap``).
2. Write clean, modular code complying with :doc:`../coding-style`.
3. Verify all checks pass via ``make check``, ``make test-all`` and ``make docs-lint``.
4. Rebase against latest upstream ``main`` to ensure a clean linear git history without merge commits.
5. Submit the pull request with a clear description referencing relevant issues or milestone targets.
