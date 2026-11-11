.. SPDX-License-Identifier: GPL-2.0-only

=======================================
Git Workflow & Collaboration Guidelines
=======================================

This document outlines the branch management, collaboration workflow and revision procedures for Keira Kernel development.

Branching Strategy
==================

.. code-block:: text

   gitGraph
       commit id: "v0.8.0"
       branch feat/ext4-extent
       checkout feat/ext4-extent
       commit id: "feat(fs): add extent tree traversal"
       commit id: "test(fs): add extent fixture tests"
       checkout main
       merge feat/ext4-extent id: "Merge branch 'feat/ext4-extent'"
       branch fix/e1000-rx-overflow
       checkout fix/e1000-rx-overflow
       commit id: "fix(net): handle e1000 rx ring buffer wrap"
       checkout main
       merge fix/e1000-rx-overflow id: "Merge branch 'fix/e1000-rx-overflow'"

* **``main``**: The protected mainline development branch. Must always compile cleanly (``0`` errors, ``0`` warnings) across both ``x86_64`` and ``i686`` targets.
* **Feature Branches**: ``feat/<scope>-<short-description>`` (e.g., ``feat/fs-ext4-extents``, ``feat/net-tls-cipher``).
* **Fix Branches**: ``fix/<scope>-<issue-description>`` (e.g., ``fix/apic-timer-overflow``, ``fix/syscall-write-bounds``).

Linear History & Rebasing
=========================

Keira enforces a linear git history on the mainline branch:

1. Always rebase your feature branch onto the latest upstream ``main`` branch before submitting.
2. Avoid merge commits in topic branches. Keep changesets squashed or grouped into logically atomic commits.
3. Verify that every commit in your branch compiles and tests independently to ensure full ``git bisect`` support.
