# SPDX-License-Identifier: GPL-2.0-only

"""Kernel figure helper directive for Sphinx documentation.

Provides the `.. kernel-figure::` directive for styled architecture diagrams.
"""

from docutils import nodes
from docutils.parsers.rst import directives
from sphinx.util.docutils import SphinxDirective


class KernelFigureDirective(SphinxDirective):
    """Renders styled architecture and sequence figures."""

    has_content = True
    required_arguments = 0
    optional_arguments = 1
    final_argument_whitespace = True
    option_spec = {
        "caption": directives.unchanged,
        "align": directives.unchanged,
    }

    def run(self):
        container = nodes.container(classes=["kernel-figure-box"])
        caption_text = self.options.get("caption") or (self.arguments[0] if self.arguments else None)

        text_content = "\n".join(self.content)
        block = nodes.literal_block(text_content, text_content)
        block["language"] = "text"
        block["classes"].append("kernel-diagram-block")
        container += block

        if caption_text:
            cap_p = nodes.paragraph(text=caption_text, classes=["kernel-figure-caption"])
            container += cap_p

        self.set_source_info(container)
        return [container]


def setup(app):
    """Register kernel-figure directive."""
    app.add_directive("kernel-figure", KernelFigureDirective)
    return {
        "version": "0.8.0",
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }

