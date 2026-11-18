#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# Generates include/generated/autoconf.h from .config

import os
import re
import sys

def main():
    if len(sys.argv) < 3:
        print("Usage: genheader.py <.config> <autoconf.h>")
        sys.exit(1)

    config_path = sys.argv[1]
    header_path = sys.argv[2]

    if not os.path.exists(config_path):
        print(f"[ERR] Config file not found: {config_path}")
        sys.exit(1)

    os.makedirs(os.path.dirname(header_path), exist_ok=True)

    macros = []
    with open(config_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, val = line.split("=", 1)
                key = key.strip()
                val = val.strip()
                if key.startswith("CONFIG_"):
                    if val == "y":
                        macros.append(f"#define {key} 1")
                    elif val.startswith('"') and val.endswith('"'):
                        macros.append(f"#define {key} {val}")
                    else:
                        macros.append(f"#define {key} {val}")

    with open(header_path, "w", encoding="utf-8") as f:
        f.write("/*\n * Automatically generated file; DO NOT EDIT.\n")
        f.write(" * Keira Kernel Configuration Header\n */\n\n")
        f.write("#ifndef _KEIRA_AUTOCONF_H\n#define _KEIRA_AUTOCONF_H\n\n")
        for m in sorted(macros):
            f.write(f"{m}\n")
        f.write("\n#endif /* _KEIRA_AUTOCONF_H */\n")

    print(f"[KCONFIG] Successfully generated {header_path} ({len(macros)} symbols)")

if __name__ == "__main__":
    main()
