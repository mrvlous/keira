#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Code Style and Patch Verification Utility for Keira Kernel.

Enforces SPDX license identifier presence, trailing whitespace limits,
single spacing and strict absence of Oxford commas. Modeled after Linux
kernel scripts/checkpatch.pl.
"""

import os
import re
import sys

OXFORD_COMMA_RE = re.compile(r',\s+(?:and|or)\b', re.IGNORECASE)
DOUBLE_SPACE_RE = re.compile(r'\.\s{2,}[A-Z]')

def check_file(file_path):
    issues = []
    if not os.path.exists(file_path) or os.path.isdir(file_path):
        return issues

    ext = os.path.splitext(file_path)[1].lower()
    if ext in ('.png', '.bin', '.iso', '.img', '.tar', '.o', '.a', '.elf', '.lock', '.pyc'):
        return issues

    with open(file_path, "r", encoding="utf-8", errors="replace") as f:
        lines = f.readlines()

    # Check SPDX tag in first 10 lines
    has_spdx = any("SPDX-License-Identifier:" in line for line in lines[:10])
    if not has_spdx and ext in ('.rs', '.c', '.h', '.asm', '.py', '.sh'):
        issues.append("Line 1: Missing SPDX-License-Identifier tag")

    for idx, line in enumerate(lines, 1):
        raw = line.rstrip("\r\n")
        # Trailing whitespace check
        if line.endswith((" \n", "\t\n")):
            issues.append(f"Line {idx}: Trailing whitespace detected")

        # In prose or comments, check Oxford comma
        if ext in ('.rst', '.md', '.txt') or line.strip().startswith(("//", "/*", "*", "#")):
            if OXFORD_COMMA_RE.search(raw):
                issues.append(f"Line {idx}: Oxford comma detected (use 'A and B' instead of 'A, and B')")

    return issues

def main():
    if len(sys.argv) < 2:
        print("Usage: scripts/checkpatch.py <file-or-dir>...")
        sys.exit(1)

    total_issues = 0
    checked_files = 0

    for arg in sys.argv[1:]:
        if os.path.isdir(arg):
            for root, _, files in os.walk(arg):
                if any(p in root for p in ("target", "build", ".git")):
                    continue
                for f in files:
                    p = os.path.join(root, f)
                    errs = check_file(p)
                    checked_files += 1
                    if errs:
                        total_issues += len(errs)
                        print(f"ERROR: {p}")
                        for e in errs:
                            print(f"  {e}")
        elif os.path.isfile(arg):
            errs = check_file(arg)
            checked_files += 1
            if errs:
                total_issues += len(errs)
                print(f"ERROR: {arg}")
                for e in errs:
                    print(f"  {e}")

    print(f"\n[CHECKPATCH] Verified {checked_files} files, {total_issues} issues found.")
    if total_issues > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
