#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# Batch Kconfig Configuration Processor (defconfig, pure_defconfig, savedefconfig)

import argparse
import os
import sys

from kconfig import Kconfig

def main():
    parser = argparse.ArgumentParser(description="Keira Kernel Kconfig Batch Processor")
    parser.add_argument("kconfig_file", nargs="?", default="Kconfig", help="Root Kconfig file")
    parser.add_argument("--defconfig", nargs="?", const="default", help="Generate default configuration")
    parser.add_argument("--pure-defconfig", action="store_true", help="Generate pure Ring 0 configuration")
    parser.add_argument("--savedefconfig", metavar="FILE", help="Save minimal defconfig")
    parser.add_argument("--oldconfig", action="store_true", help="Validate existing .config")
    parser.add_argument("--autoconf", metavar="FILE", help="Generate autoconf C header")
    parser.add_argument("-o", "--output", default=".config", help="Output .config path")

    args = parser.parse_args()

    kconf = Kconfig(args.kconfig_file)

    if args.defconfig is not None:
        defconfig_file = args.defconfig
        if defconfig_file in ("default", "Kconfig", ""):
            target_arch = os.environ.get("ARCH", "x86_64")
            if target_arch in ("i386", "i686"):
                target_arch = "i686"
            else:
                target_arch = "x86_64"
            candidate = f"arch/x86/configs/{target_arch}_defconfig"
            if os.path.exists(candidate):
                defconfig_file = candidate
            elif os.path.exists(".example.config"):
                defconfig_file = ".example.config"
        if os.path.exists(defconfig_file):
            kconf.load_config(defconfig_file)
        kconf.write_config(args.output)
        print(f"[KCONFIG] Loaded {defconfig_file} -> {args.output}")

    elif args.pure_defconfig:
        candidate = "arch/x86/configs/pure_defconfig"
        if os.path.exists(candidate):
            kconf.load_config(candidate)
        else:
            if os.path.exists(".example.config"):
                kconf.load_config(".example.config")
            if "USERLAND" in kconf.symbols:
                kconf.symbols["USERLAND"].value = False
        kconf.write_config(args.output)
        print(f"[KCONFIG] Loaded pure Ring 0 configuration -> {args.output}")

    elif args.savedefconfig:
        kconf.load_config(args.output)
        with open(args.savedefconfig, "w", encoding="utf-8") as f:
            for name, sym in sorted(kconf.symbols.items()):
                val = sym.get_value(kconf)
                def_val = sym.get_effective_default(kconf)
                if val != def_val:
                    if sym.type == "bool":
                        if val:
                            f.write(f"CONFIG_{name}=y\n")
                        else:
                            f.write(f"# CONFIG_{name} is not set\n")
                    else:
                        f.write(f"CONFIG_{name}={val}\n")
        print(f"[KCONFIG] Saved minimal configuration to {args.savedefconfig}")

    elif args.oldconfig:
        if os.path.exists(args.output):
            kconf.load_config(args.output)
        kconf.write_config(args.output)
        print(f"[KCONFIG] Validated {args.output}")

    if args.autoconf:
        kconf.load_config(args.output)
        kconf.write_autoconf(args.autoconf)
        print(f"[KCONFIG] Generated {args.autoconf}")

if __name__ == "__main__":
    main()
