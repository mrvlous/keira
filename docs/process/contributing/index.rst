.. SPDX-License-Identifier: GPL-2.0-only


Contributor Guide & Technical Principles
========================================

Welcome to the technical contributor guide for the Keira Kernel project. This modular documentation section outlines the architectural invariants, engineering workflow, git commit standards and local quality gate verification requirements for kernel contributors.

Keira is an autonomous, freestanding Ring 0 monolithic kernel built from scratch in safe Rust, Assembly and C. Development is guided by strict systems programming discipline modeled after the Linux Kernel and industry best practices.

Contributor Documentation Map
=============================

.. toctree::
   :maxdepth: 2

   overview
   development-workflow
   submitting-patches
   quality-checklist

Subsystem Scope & Maintainers
=============================

Before beginning work on a patch series:

* Identify the appropriate subsystem maintainer in ``MAINTAINERS``.
* Verify code compliance against the :doc:`../coding-style` manual.
* Ensure all changes compile cleanly without errors or warnings across both ``x86_64`` and ``i686`` architectures.
