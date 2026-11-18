#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# Subsystem Maintainer Query Utility for Keira Kernel
# Modeled after Linux kernel scripts/get_maintainer.pl

import fnmatch
import os
import sys

def parse_maintainers(file_path="MAINTAINERS"):
    if not os.path.exists(file_path):
        return []

    sections = []
    current_title = None
    current_entries = []

    with open(file_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith(("P:", "M:", "W:", "S:", "F:")):
                key, val = line.split(":", 1)
                current_entries.append((key.strip(), val.strip()))
            else:
                if current_title and current_entries:
                    sections.append((current_title, current_entries))
                    current_entries = []
                current_title = line

    if current_title and current_entries:
        sections.append((current_title, current_entries))

    return sections

def find_maintainers(target_path, sections):
    matched = []
    for title, entries in sections:
        files = [val for key, val in entries if key == "F"]
        for pattern in files:
            norm_pattern = pattern.rstrip("/")
            norm_target = target_path.lstrip("./").rstrip("/")
            if pattern == "*" or fnmatch.fnmatch(norm_target, norm_pattern) or norm_target.startswith(norm_pattern + "/"):
                matched.append((title, entries))
                break
    return matched

def main():
    if len(sys.argv) < 2:
        print("Usage: scripts/get_maintainer.py <file-path>")
        sys.exit(1)

    target = sys.argv[1]
    sections = parse_maintainers()
    matches = find_maintainers(target, sections)

    seen = set()
    for title, entries in matches:
        person = next((val for k, val in entries if k == "P"), "Unknown")
        email = next((val for k, val in entries if k == "M"), "")
        status = next((val for k, val in entries if k == "S"), "")
        key = (person, email)
        if key not in seen:
            seen.add(key)
            if email:
                print(f"{person} <{email}> ({title}, {status})")
            else:
                print(f"{person} ({title}, {status})")

if __name__ == "__main__":
    main()
