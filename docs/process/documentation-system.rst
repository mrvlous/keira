.. SPDX-License-Identifier: GPL-2.0-only

====================================
Sphinx Documentation Build Subsystem
====================================

The Keira Kernel documentation architecture is modeled directly on the upstream
Linux Kernel documentation infrastructure (``Documentation/`` in ``torvalds/linux``).
It provides a modular, reproducible and scalable technical reference suite across
16 first-class subsystem domains.

1. Architectural Hierarchy & Subsystems
=======================================

To prevent root directory clutter and isolate build orchestration from documentation
content, all tooling, assets and dependencies are organized into dedicated directories:

.. code-block:: text

   docs/
   ├── Makefile                   # Documentation build orchestration
   ├── conf.py                    # Master Sphinx configuration & extension loader
   ├── sphinx/                    # Sphinx build extensions & toolchain requirements
   │   ├── requirements.txt       # Canonical Python dependency specifications
   │   ├── check_docs.py          # Standalone style & SPDX compliance linter
   │   ├── kernel_include.py      # Custom .. kernel-include:: directive
   │   ├── maintainers_include.py # Automatic MAINTAINERS table parser
   │   ├── load_config.py         # Dynamic kernel configuration extractor
   │   ├── kfigure.py             # Custom figure and diagram helpers
   │   └── templates/             # Custom Jinja2 layout overrides
   ├── sphinx-static/             # Static CSS & JavaScript assets
   │   ├── theme_overrides.css    # Clean Linux-grade styling overrides
   │   ├── theme_rtd_colors.css   # Contrast & palette tokens
   │   ├── custom.css             # Supplementary component styles
   │   └── custom.js              # Keyboard shortcuts & interactive navigation
   ├── sphinx-include/            # Shared ReST boilerplate snippets
   │   ├── spdx-license.rst       # Canonical licensing notice
   │   ├── disclaimer.rst         # Technical specification disclaimer
   │   ├── coding-conventions.rst # Quick coding standard summary
   │   └── architecture-summary.rst # Dual-architecture overview
   └── <domain>/                  # 16 modular documentation domains

2. Python Dependencies & Environment Setup
==========================================

In accordance with the Linux Kernel documentation standard, Python dependencies are
strictly maintained in ``docs/sphinx/requirements.txt`` rather than in the root of
the documentation tree. This encapsulation preserves a clean documentation root
dedicated exclusively to books, guides and configuration entry points.

Recommended Virtual Environment Setup:
--------------------------------------

To avoid conflicts with host distribution packages, always build documentation within
an isolated Python virtual environment:

.. code-block:: bash

   # 1. Create a local virtual environment
   python3 -m venv .venv

   # 2. Activate the virtual environment
   source .venv/bin/activate

   # 3. Install pinned Sphinx dependencies
   pip install -r docs/sphinx/requirements.txt

Core Dependencies (``docs/sphinx/requirements.txt``):
-----------------------------------------------------

* ``sphinx>=7.2.0``: Master documentation generator.
* ``sphinx-rtd-theme>=2.0.0``: Standard ReadTheDocs theme with custom color tokens.

3. Documentation Make Targets
=============================

The documentation system can be invoked from the repository root or directly
within ``docs/``:

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Make Target
     - Description
   * - ``make docs``
     - Build complete Sphinx HTML documentation in ``docs/_build/html/``
   * - ``make docs-lint``
     - Verify SPDX headers, title lengths and zero Oxford commas across all files
   * - ``make docs-linkcheck``
     - Verify external hyperlink reachability and validity
   * - ``make docs-info``
     - Display Sphinx build environment, Python interpreter and source page count
   * - ``make pdfdocs``
     - Compile PDF technical manual via Sphinx LaTeX builder (requires pdflatex)
   * - ``make docs-clean``
     - Remove generated build artifacts in ``docs/_build/``

4. ReST & Documentation Coding Standards
========================================

Every documentation file must adhere strictly to Keira's documentation guidelines:

1. **SPDX Header**: Line 1 must strictly be ``.. SPDX-License-Identifier: GPL-2.0-only`` followed by an empty line.
2. **Title Heading Underlines**: Title underline and overline characters must exactly match the character length of the title text.
3. **Grammar & Style**: Strictly 100% formal English. Omit the Oxford comma before "and" and "or" across all prose.
4. **Standalone Includes**: Shared boilerplate must reside in ``docs/sphinx-include/`` and be excluded from standalone rendering via ``exclude_patterns`` in ``conf.py``.
