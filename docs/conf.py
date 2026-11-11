# SPDX-License-Identifier: GPL-2.0-only

"""Sphinx configuration for Keira Kernel technical documentation.

Modeled after the Linux Kernel documentation architecture (Documentation/conf.py,
Documentation/sphinx/, Documentation/sphinx-static/ and Documentation/sphinx-include/),
incorporating modular extensions, custom kernel inclusion directives, professional
styling and REUSE 3.0 licensing compliance.
"""

import os
import sys
from pathlib import Path

# Add docs/sphinx to Python path for custom extensions
DOCS_DIR = Path(__file__).resolve().parent
SPHINX_DIR = DOCS_DIR / "sphinx"
sys.path.insert(0, str(SPHINX_DIR))

# Dynamic version extraction via load_config
try:
    from load_config import get_kernel_version

    _kernel_version = get_kernel_version()
except Exception:
    _kernel_version = "0.8.0"

# Project Information
project = "Keira Kernel"
copyright = "2026, Moh. Ananda Firmansyah Putra"
author = "Moh. Ananda Firmansyah Putra"
version = _kernel_version
release = _kernel_version

# General Sphinx Configuration
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.viewcode",
    "sphinx.ext.todo",
    "sphinx_rtd_theme",
    # Custom Keira Linux-grade Sphinx Extensions
    "load_config",
    "kernel_include",
    "maintainers_include",
    "kfigure",
]

# Source & Template Paths
templates_path = ["sphinx/templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store", "sphinx-include"]
source_suffix = ".rst"
master_doc = "index"
language = "en"

# HTML Output Configuration
html_theme = "sphinx_rtd_theme"
html_title = "Keira Kernel Documentation"
html_short_title = "Keira"

html_theme_options = {
    "collapse_navigation": False,
    "sticky_navigation": True,
    "navigation_depth": 4,
    "includehidden": True,
    "titles_only": False,
    "logo_only": False,
    "prev_next_buttons_location": "bottom",
    "style_external_links": False,
}

# Static Assets (Linux Documentation/sphinx-static standard)
html_static_path = ["sphinx-static"]
html_css_files = [
    "theme_overrides.css",
    "theme_rtd_colors.css",
    "custom.css",
]
html_js_files = [
    "custom.js",
]

# HTML Presentation Context
html_show_sourcelink = False
html_show_sphinx = True
html_copy_source = False

# Pygments Syntax Highlighting Style
pygments_style = "monokai"

# Todo Extension Configuration
todo_include_todos = True
