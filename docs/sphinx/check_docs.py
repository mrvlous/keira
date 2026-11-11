#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Documentation Style & License Compliance Checker for Keira Kernel.

Validates:
1. Line 1 must strictly be `.. SPDX-License-Identifier: GPL-2.0-only`.
2. Line 2 must be an empty line.
3. No multiple trailing newlines at EOF, single newline required.
4. No trailing whitespace on any line.
5. Zero Oxford commas (", and " or ", or ").
6. Title overline and underline character lengths must match title string length.
"""

import sys
from pathlib import Path


def check_file(path: Path) -> list[str]:
    """Check a single .rst file for compliance."""
    errors = []
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    if not lines or lines[0] != ".. SPDX-License-Identifier: GPL-2.0-only":
        errors.append("Line 1 missing `.. SPDX-License-Identifier: GPL-2.0-only`")

    if len(lines) > 1 and lines[1] != "":
        errors.append("Line 2 must be an empty line following SPDX header")

    if text.endswith("\n\n"):
        errors.append("Multiple trailing blank lines at end of file")
    elif not text.endswith("\n"):
        errors.append("Missing trailing newline at end of file")

    in_code_block = False
    code_indent = 0

    for idx, line in enumerate(lines):
        line_num = idx + 1

        if line.rstrip() != line:
            errors.append(f"Line {line_num}: Trailing whitespace detected")

        if not in_code_block and (", and " in line or ", or " in line):
            errors.append(f"Line {line_num}: Contains Oxford comma (', and ' or ', or ')")

        # Track code blocks to avoid false title match warnings
        stripped = line.strip()
        if stripped.startswith(".. code-block::") or stripped == "::":
            in_code_block = True
            indent = len(line) - len(line.lstrip())
            code_indent = indent
            continue

        if in_code_block:
            if stripped == "":
                continue
            cur_indent = len(line) - len(line.lstrip())
            if cur_indent <= code_indent:
                in_code_block = False

        if not in_code_block and idx > 0 and idx < len(lines) - 1:
            prev_line = lines[idx - 1]
            next_line = lines[idx + 1]
            if (
                len(set(prev_line)) == 1
                and prev_line[0] in "=-~^"
                and len(set(next_line)) == 1
                and next_line[0] == prev_line[0]
            ):
                title_len = len(line)
                if len(prev_line) != title_len or len(next_line) != title_len:
                    errors.append(
                        f"Line {line_num}: Title length ({title_len}) mismatch overline ({len(prev_line)}) / underline ({len(next_line)})"
                    )

    return errors


def main() -> int:
    """Scan docs directory and report violations."""
    docs_root = Path(__file__).resolve().parent.parent
    errors = {}
    count = 0

    for path in sorted(docs_root.rglob("*.rst")):
        if "_build" in path.parts:
            continue
        count += 1
        rel_path = path.relative_to(docs_root)
        errs = check_file(path)
        if errs:
            errors[str(rel_path)] = errs

    print(f"Scanned {count} documentation files.")
    if errors:
        print(f"FAIL: {len(errors)} files failed compliance checks:", file=sys.stderr)
        for f, errs in errors.items():
            print(f"  {f}:", file=sys.stderr)
            for err in errs:
                print(f"    - {err}", file=sys.stderr)
        return 1

    print("SUCCESS: 100% compliant with Keira documentation style guidelines.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
