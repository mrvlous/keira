# SPDX-License-Identifier: GPL-2.0-only

"""Kernel and Sphinx file inclusion directive for Keira documentation.

Modeled after Linux Documentation/sphinx/kernel_include.py. Provides the
`.. kernel-include::` and `.. sphinx-include::` directives to safely include
kernel sources and shared documentation snippets.
"""

from pathlib import Path
from docutils import nodes
from docutils.parsers.rst import directives
from sphinx.util.docutils import SphinxDirective


class KernelIncludeDirective(SphinxDirective):
    """Include a kernel source file or documentation snippet."""

    has_content = False
    required_arguments = 1
    optional_arguments = 0
    final_argument_whitespace = True
    option_spec = {
        "language": directives.unchanged,
        "linenos": directives.flag,
        "start-at": directives.unchanged,
        "end-before": directives.unchanged,
    }

    def run(self):
        rel_path = self.arguments[0].strip()
        repo_root = Path(__file__).resolve().parent.parent.parent
        docs_include = repo_root / "docs" / "sphinx-include"

        # Check in sphinx-include first if simple filename, else repo root
        if (docs_include / rel_path).is_file():
            target_path = (docs_include / rel_path).resolve()
        else:
            target_path = (repo_root / rel_path).resolve()

        if not str(target_path).startswith(str(repo_root)):
            raise self.severe(f"Path outside kernel repository: {rel_path}")

        if not target_path.is_file():
            raise self.warning(f"Include file not found: {rel_path}")

        try:
            content = target_path.read_text(encoding="utf-8")
        except Exception as e:
            raise self.severe(f"Error reading {rel_path}: {e}")

        lines = content.splitlines()
        start_at = self.options.get("start-at")
        end_before = self.options.get("end-before")

        if start_at:
            for idx, line in enumerate(lines):
                if start_at in line:
                    lines = lines[idx:]
                    break

        if end_before:
            for idx, line in enumerate(lines):
                if end_before in line:
                    lines = lines[:idx]
                    break

        filtered_text = "\n".join(lines)
        language = self.options.get("language")
        if not language:
            suffix = target_path.suffix.lower()
            lang_map = {
                ".rs": "rust",
                ".c": "c",
                ".h": "c",
                ".asm": "nasm",
                ".inc": "nasm",
                ".ld": "text",
                ".toml": "toml",
                ".py": "python",
                ".sh": "bash",
                ".md": "markdown",
                ".rst": "rst",
            }
            language = lang_map.get(suffix, "text")

        node = nodes.literal_block(filtered_text, filtered_text)
        node["language"] = language
        if "linenos" in self.options:
            node["linenos"] = True

        self.set_source_info(node)
        return [node]


def setup(app):
    """Register kernel-include and sphinx-include directives."""
    app.add_directive("kernel-include", KernelIncludeDirective)
    app.add_directive("sphinx-include", KernelIncludeDirective)
    return {
        "version": "0.8.0",
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }
