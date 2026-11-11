# SPDX-License-Identifier: GPL-2.0-only

"""Maintainers file parser directive for Sphinx documentation.

Provides the `.. maintainers-include::` directive to parse the Linux-format
MAINTAINERS file and render it as a structured table.
"""

from pathlib import Path
from docutils import nodes
from sphinx.util.docutils import SphinxDirective


class MaintainersIncludeDirective(SphinxDirective):
    """Renders MAINTAINERS entries as a structured table."""

    has_content = False
    required_arguments = 0
    optional_arguments = 0

    def run(self):
        repo_root = Path(__file__).resolve().parent.parent.parent
        maintainers_path = repo_root / "MAINTAINERS"

        if not maintainers_path.exists():
            return [nodes.paragraph(text="MAINTAINERS file not found.")]

        lines = maintainers_path.read_text(encoding="utf-8").splitlines()
        entries = []
        current = None

        for line in lines:
            line_str = line.strip()
            if not line_str or line_str.startswith("#"):
                continue
            if line_str.startswith("The fields are:") or line_str.startswith("List of maintainers"):
                continue
            if line_str.startswith("P:") or line_str.startswith("M:") or line_str.startswith("W:") or line_str.startswith("S:") or line_str.startswith("F:"):
                parts = line_str.split(":", 1)
                tag = parts[0].strip()
                val = parts[1].strip() if len(parts) > 1 else ""
                if current and tag in current:
                    current[tag].append(val)
            else:
                if current:
                    entries.append(current)
                current = {
                    "name": line_str,
                    "P": [],
                    "M": [],
                    "W": [],
                    "S": [],
                    "F": [],
                }
        if current:
            entries.append(current)

        table = nodes.table()
        tgroup = nodes.tgroup(cols=4)
        table += tgroup
        tgroup += nodes.colspec(colwidth=25)
        tgroup += nodes.colspec(colwidth=25)
        tgroup += nodes.colspec(colwidth=15)
        tgroup += nodes.colspec(colwidth=35)

        thead = nodes.thead()
        tgroup += thead
        head_row = nodes.row()
        for title in ["Subsystem", "Maintainer", "Status", "Matching Files"]:
            entry = nodes.entry()
            entry += nodes.paragraph(text=title)
            head_row += entry
        thead += head_row

        tbody = nodes.tbody()
        tgroup += tbody
        for item in entries:
            row = nodes.row()

            e1 = nodes.entry()
            e1 += nodes.paragraph(text=item["name"])
            row += e1

            e2 = nodes.entry()
            m_text = ", ".join(item["P"]) if item["P"] else "Moh. Ananda Firmansyah Putra"
            if item["M"]:
                m_text += f" <{', '.join(item['M'])}>"
            e2 += nodes.paragraph(text=m_text)
            row += e2

            e3 = nodes.entry()
            s_text = ", ".join(item["S"]) if item["S"] else "Maintained"
            e3 += nodes.paragraph(text=s_text)
            row += e3

            e4 = nodes.entry()
            f_text = ", ".join(item["F"]) if item["F"] else "*"
            e4 += nodes.paragraph(text=f_text)
            row += e4

            tbody += row

        self.set_source_info(table)
        return [table]


def setup(app):
    """Register maintainers-include directive."""
    app.add_directive("maintainers-include", MaintainersIncludeDirective)
    return {
        "version": "0.8.0",
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }

