#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Code Style, Commit Message and Patch Verification Utility for Keira Kernel.

Enforces SPDX license identifier presence, trailing whitespace limits,
single spacing, conventional commit formatting and strict absence of
Oxford commas. Modeled after Linux kernel scripts/checkpatch.pl.
"""

import argparse
import os
import re
import stat
import subprocess
import sys
import tempfile
import unittest

OXFORD_COMMA_RE = re.compile(r',\s+(?:and|or)\b', re.IGNORECASE)
DOUBLE_SPACE_RE = re.compile(r'\.\s{2,}[A-Z]')
RELEASE_COMMIT_RE = re.compile(r'^Keira \d+\.\d+\.\d+(?:-[A-Za-z0-9.]+)?$')
CONVENTIONAL_SUBJECT_RE = re.compile(r'^([a-z]+)\(([a-z0-9_-]+)\): (.+)$')
MISSING_SCOPE_SUBJECT_RE = re.compile(r'^([a-z]+): (.+)$')
SCISSORS_LINE = "# ------------------------ >8 ------------------------"
ALLOWED_COMMIT_TYPES = (
    "feat",
    "fix",
    "refactor",
    "perf",
    "docs",
    "style",
    "test",
    "build",
    "chore",
)
IGNORED_EXTENSIONS = (
    ".png",
    ".bin",
    ".iso",
    ".img",
    ".tar",
    ".o",
    ".a",
    ".elf",
    ".lock",
    ".pyc",
)
SPDX_REQUIRED_EXTENSIONS = (".rs", ".c", ".h", ".asm", ".py", ".sh")
PROSE_EXTENSIONS = (".rst", ".md", ".txt")

def check_file(file_path):
    """Validate a single source or documentation file for style rules."""
    issues = []
    if not os.path.exists(file_path) or os.path.isdir(file_path):
        return issues

    ext = os.path.splitext(file_path)[1].lower()
    if ext in IGNORED_EXTENSIONS:
        return issues

    with open(file_path, "r", encoding="utf-8", errors="replace") as handle:
        lines = handle.readlines()

    has_spdx = any("SPDX-License-Identifier:" in line for line in lines[:10])
    if not has_spdx and ext in SPDX_REQUIRED_EXTENSIONS:
        issues.append("Line 1: Missing SPDX-License-Identifier tag")

    for idx, line in enumerate(lines, 1):
        raw = line.rstrip("\r\n")
        if line.endswith((" \n", "\t\n")):
            issues.append(f"Line {idx}: Trailing whitespace detected")

        if ext in PROSE_EXTENSIONS or line.strip().startswith(("//", "/*", "*", "#")):
            if OXFORD_COMMA_RE.search(raw):
                issues.append(
                    f"Line {idx}: Oxford comma detected (use 'A and B' instead of 'A, and B')"
                )

    return issues

def extract_commit_lines(message_text):
    """Strip Git comment and scissor diff lines from a commit message."""
    extracted = []
    for raw_line in message_text.splitlines():
        if raw_line.startswith(SCISSORS_LINE):
            break
        if raw_line.startswith("#"):
            continue
        extracted.append(raw_line)

    while extracted and not extracted[-1].strip():
        extracted.pop()
    return extracted

def check_commit_subject(subject):
    """Validate the first line of a commit message against project conventions."""
    issues = []
    if subject != subject.strip():
        issues.append("Subject: Leading or trailing whitespace detected")
        return issues

    if RELEASE_COMMIT_RE.match(subject):
        return issues

    if subject.startswith("Merge ") or subject.startswith('Revert "'):
        return issues

    match = CONVENTIONAL_SUBJECT_RE.match(subject)
    if not match:
        no_scope = MISSING_SCOPE_SUBJECT_RE.match(subject)
        if no_scope and no_scope.group(1) in ALLOWED_COMMIT_TYPES:
            issues.append(
                "Subject: Missing commit scope (expected '<type>(<scope>): <summary>')"
            )
        else:
            allowed = ", ".join(ALLOWED_COMMIT_TYPES)
            issues.append(
                f"Subject: Invalid format (expected '<type>(<scope>): <summary>' "
                f"with type in [{allowed}] or 'Keira <version>')"
            )
        return issues

    commit_type, _, summary = match.groups()
    if commit_type not in ALLOWED_COMMIT_TYPES:
        allowed = ", ".join(ALLOWED_COMMIT_TYPES)
        issues.append(f"Subject: Unsupported commit type '{commit_type}' (allowed: {allowed})")

    if summary != summary.lstrip():
        issues.append("Subject: Multiple spaces after colon")

    if summary.endswith("."):
        issues.append("Subject: Omit trailing period from commit subject")

    if re.search(r'(?:^|\s)&(?:\s|$)', summary):
        issues.append("Subject: Spell out 'and' instead of using '&'")

    return issues

def check_commit_message(message_text):
    """Validate commit message format, grammar and Oxford comma rules."""
    issues = []
    lines = extract_commit_lines(message_text)
    if not any(line.strip() for line in lines):
        return ["Commit message is empty"]

    if not lines[0].strip():
        issues.append("Line 1: Commit message must start with a non-empty subject line")
        return issues

    issues.extend(check_commit_subject(lines[0]))

    if len(lines) > 1 and lines[1].strip():
        issues.append("Line 2: Separate commit subject and body with a blank line")

    for idx, line in enumerate(lines, 1):
        if line != line.rstrip(" \t"):
            issues.append(f"Line {idx}: Trailing whitespace detected")
        if OXFORD_COMMA_RE.search(line):
            issues.append(
                f"Line {idx}: Oxford comma detected (use 'A and B' instead of 'A, and B')"
            )
        if DOUBLE_SPACE_RE.search(line):
            issues.append(f"Line {idx}: Double space after period detected")

    return issues

def check_commit_msg_file(file_path):
    """Read and validate a Git COMMIT_EDITMSG file on disk."""
    if not os.path.isfile(file_path):
        print(f"ERROR: Commit message file not found: {file_path}", file=sys.stderr)
        return 1

    with open(file_path, "r", encoding="utf-8", errors="replace") as handle:
        content = handle.read()

    issues = check_commit_message(content)
    if issues:
        print(f"ERROR: Commit message check failed ({file_path})")
        for issue in issues:
            print(f"  {issue}")
        return 1

    print("[CHECKPATCH] Commit message complies with project style rules.")
    return 0

def check_git_log(rev_range):
    """Validate commit messages across a Git revision or range specification."""
    cmd = ["git", "log", "--format=%H%x1e%B%x1d", rev_range]
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=False)
    if res.returncode != 0:
        print(f"ERROR: git log failed: {res.stderr.strip()}", file=sys.stderr)
        return 1

    records = [rec for rec in res.stdout.split("\x1d") if rec.strip()]
    total_issues = 0
    for rec in records:
        commit_hash, body = rec.lstrip("\n").split("\x1e", 1)
        short_hash = commit_hash[:7]
        issues = check_commit_message(body)
        if issues:
            total_issues += len(issues)
            subject = body.splitlines()[0] if body.splitlines() else ""
            print(f"ERROR: {short_hash} ({subject})")
            for issue in issues:
                print(f"  {issue}")

    print(
        f"\n[CHECKPATCH] Verified {len(records)} commits, {total_issues} issues found."
    )
    return 1 if total_issues > 0 else 0

def install_commit_msg_hook():
    """Install the local Git commit-msg hook backed by scripts/checkpatch.py."""
    res = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks/commit-msg"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if res.returncode != 0:
        print(f"ERROR: Unable to locate git hooks directory: {res.stderr.strip()}", file=sys.stderr)
        return 1

    hook_path = res.stdout.strip()
    os.makedirs(os.path.dirname(os.path.abspath(hook_path)), exist_ok=True)
    hook_script = (
        "#!/bin/sh\n"
        "# SPDX-License-Identifier: GPL-2.0-only\n"
        "set -eu\n"
        'ROOT_DIR=$(git rev-parse --show-toplevel)\n'
        'exec python3 "$ROOT_DIR/scripts/checkpatch.py" --commit-msg "$1"\n'
    )
    with open(hook_path, "w", encoding="utf-8") as handle:
        handle.write(hook_script)

    mode = os.stat(hook_path).st_mode
    os.chmod(hook_path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    print(f"[CHECKPATCH] Installed Git commit-msg hook at {hook_path}")
    return 0

def check_paths(paths):
    """Recursively validate all provided file and directory paths."""
    total_issues = 0
    checked_files = 0

    for arg in paths:
        if os.path.isdir(arg):
            for root, _, files in os.walk(arg):
                if any(part in root for part in ("target", "build", ".git")):
                    continue
                for filename in sorted(files):
                    path = os.path.join(root, filename)
                    errs = check_file(path)
                    checked_files += 1
                    if errs:
                        total_issues += len(errs)
                        print(f"ERROR: {path}")
                        for err in errs:
                            print(f"  {err}")
        elif os.path.isfile(arg):
            errs = check_file(arg)
            checked_files += 1
            if errs:
                total_issues += len(errs)
                print(f"ERROR: {arg}")
                for err in errs:
                    print(f"  {err}")

    print(f"\n[CHECKPATCH] Verified {checked_files} files, {total_issues} issues found.")
    return 1 if total_issues > 0 else 0

def run_self_tests():
    """Execute built-in unit tests for file and commit message verification."""

    class TestCheckpatch(unittest.TestCase):
        """Unit test cases for checkpatch style and commit message validation."""

        def test_valid_conventional_commit(self):
            msg = "docs(sync): synchronize fhs, packaging, procfs and syscall registry docs\n"
            self.assertEqual(check_commit_message(msg), [])

        def test_valid_release_and_body(self):
            msg = (
                "Keira 0.9.0\n\n"
                "Stabilize buddy allocator, workqueues and shell pipelines.\n"
            )
            self.assertEqual(check_commit_message(msg), [])

        def test_reject_oxford_comma_in_subject_and_body(self):
            issues_sub = check_commit_message(
                "docs(sync): sync FHS, packaging, procfs, and syscall registry docs\n"
            )
            self.assertTrue(any("Oxford comma" in i for i in issues_sub))
            issues_body = check_commit_message(
                "fix(mem): unify buddy free path\n\nCovers DMA, normal, or high zones.\n"
            )
            self.assertTrue(any("Oxford comma" in i for i in issues_body))

        def test_reject_missing_scope_and_ampersand(self):
            issues_scope = check_commit_message("docs: sync coding style guides\n")
            self.assertTrue(any("Missing commit scope" in i for i in issues_scope))
            issues_amp = check_commit_message("feat(net): add dhcp & dns client\n")
            self.assertTrue(any("Spell out 'and'" in i for i in issues_amp))

        def test_ignore_git_comments_and_scissors(self):
            msg = (
                "fix(task): wake idle kernel threads on pipe write\n"
                "# Please enter the commit message for your changes.\n"
                f"{SCISSORS_LINE}\n"
                "+ // Ignored diff lines like A, and B\n"
            )
            self.assertEqual(check_commit_message(msg), [])

        def test_check_file_rejects_oxford_comma(self):
            with tempfile.NamedTemporaryFile("w", suffix=".rst", delete=False) as tmp:
                tmp.write("Header, body, and footer.\n")
                tmp_path = tmp.name
            try:
                issues = check_file(tmp_path)
                self.assertEqual(len(issues), 1)
                self.assertIn("Oxford comma", issues[0])
            finally:
                os.unlink(tmp_path)

    suite = unittest.TestLoader().loadTestsFromTestCase(TestCheckpatch)
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    return 0 if result.wasSuccessful() else 1

def main():
    """Parse CLI arguments and dispatch checkpatch validation modes."""
    parser = argparse.ArgumentParser(
        description="Code Style, Commit Message and Patch Verification Utility for Keira Kernel"
    )
    parser.add_argument("paths", nargs="*", help="Source files or directories to verify")
    parser.add_argument(
        "--commit-msg",
        metavar="FILE",
        help="Validate a commit message file (used by Git commit-msg hook)",
    )
    parser.add_argument(
        "--git-log",
        metavar="REV_RANGE",
        help="Validate existing commit messages across a Git revision or range",
    )
    parser.add_argument(
        "--install-hook",
        action="store_true",
        help="Install local Git commit-msg hook backed by this script",
    )
    parser.add_argument(
        "--test",
        action="store_true",
        help="Execute built-in unit tests",
    )

    args = parser.parse_args()

    if args.test:
        return run_self_tests()
    if args.install_hook:
        return install_commit_msg_hook()
    if args.commit_msg:
        return check_commit_msg_file(args.commit_msg)
    if args.git_log:
        return check_git_log(args.git_log)
    if args.paths:
        return check_paths(args.paths)

    parser.print_help()
    return 1

if __name__ == "__main__":
    sys.exit(main())
