#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Unit Test Suite for Kconfig Engine and Kbuild Primitives.

Validates Kconfig grammar parsing, symbol evaluation, dependency resolution,
configuration loading, autoconf generation and Kbuild helper macros. Modeled
after Linux kernel scripts/kconfig unit tests.
"""

import os
import shutil
import subprocess
import tempfile
import unittest

from kconfig import Choice, Kconfig, Menu, Symbol

class TestKconfigModel(unittest.TestCase):
    """Tests symbol, choice and menu data structures."""

    def test_symbol_defaults_and_values(self):
        sym = Symbol("TEST_SYM")
        sym.type = "bool"
        sym.defaults.append((True, None))

        kconf = Kconfig.__new__(Kconfig)
        kconf.symbols = {"TEST_SYM": sym}

        self.assertTrue(sym.get_effective_default(kconf))
        self.assertTrue(sym.get_value(kconf))

        sym.value = False
        self.assertFalse(sym.get_value(kconf))

    def test_symbol_int_and_string_types(self):
        kconf = Kconfig.__new__(Kconfig)
        kconf.symbols = {}

        sym_int = Symbol("MAX_ITEMS")
        sym_int.type = "int"
        sym_int.defaults.append((16, None))
        kconf.symbols["MAX_ITEMS"] = sym_int

        self.assertEqual(sym_int.get_value(kconf), 16)

        sym_str = Symbol("HOSTNAME")
        sym_str.type = "string"
        sym_str.defaults.append(("keira-box", None))
        kconf.symbols["HOSTNAME"] = sym_str

        self.assertEqual(sym_str.get_value(kconf), "keira-box")

    def test_choice_selection(self):
        kconf = Kconfig.__new__(Kconfig)
        kconf.symbols = {}

        choice = Choice(prompt="Select CPU")
        sym_a = Symbol("CPU_A")
        sym_b = Symbol("CPU_B")
        choice.items = [sym_a, sym_b]
        choice.default_name = "CPU_B"

        kconf.symbols["CPU_A"] = sym_a
        kconf.symbols["CPU_B"] = sym_b

        selected = choice.get_selected(kconf)
        self.assertEqual(selected, sym_b)

        sym_a.value = True
        selected = choice.get_selected(kconf)
        self.assertEqual(selected, sym_a)

    def test_visibility_with_depends_on(self):
        kconf = Kconfig.__new__(Kconfig)
        kconf.symbols = {}
        kconf.eval_condition = lambda cond: cond == "DEP_OK"

        sym = Symbol("FEATURE")
        sym.depends_on.append("DEP_OK")
        self.assertTrue(sym.is_visible(kconf))

        sym.depends_on = ["DEP_FAIL"]
        self.assertFalse(sym.is_visible(kconf))


class TestKconfigGrammarParsing(unittest.TestCase):
    """Tests parsing the official root Kconfig and subsystem files."""

    @classmethod
    def setUpClass(cls):
        cls.kconf = Kconfig("Kconfig")

    def test_mainmenu_title(self):
        self.assertEqual(self.kconf.mainmenu, "Keira Kernel Configuration")

    def test_core_symbols_present(self):
        required_symbols = [
            "ARCH_X86_64",
            "ARCH_I686",
            "SMP",
            "MAX_CPUS",
            "ASLR",
            "W_XOR_X",
            "SCHED_WORK_STEALING",
            "IO_URING",
            "FS_FAT",
            "FS_EXT4",
            "FS_PROCFS",
            "NET_STACK",
            "NET_E1000",
            "NET_SOCKETS",
            "RLIMIT",
            "CRYPTO_ENTROPY",
            "USERLAND",
            "USERLAND_COMPILER_KCC",
            "USERLAND_TESTS",
            "DISK_SIZE",
            "QEMU_MEM",
            "QEMU_CPUS",
            "VERBOSE",
        ]
        for sym_name in required_symbols:
            self.assertIn(sym_name, self.kconf.symbols, f"Symbol {sym_name} missing from Kconfig")

    def test_condition_evaluation(self):
        self.kconf.symbols["SMP"].value = True
        self.assertTrue(self.kconf.eval_condition("SMP"))
        self.assertFalse(self.kconf.eval_condition("!SMP"))

        self.kconf.symbols["USERLAND"].value = False
        self.assertFalse(self.kconf.eval_condition("USERLAND"))
        self.assertTrue(self.kconf.eval_condition("!USERLAND"))
        self.assertTrue(self.kconf.eval_condition("SMP && !USERLAND"))
        self.assertFalse(self.kconf.eval_condition("SMP && USERLAND"))


class TestDefconfigProfiles(unittest.TestCase):
    """Validates baseline defconfig profiles and generated configurations."""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_x86_64_defconfig_loading(self):
        kconf = Kconfig("Kconfig")
        kconf.load_config("arch/x86/configs/x86_64_defconfig")

        self.assertTrue(kconf.symbols["ARCH_X86_64"].get_value(kconf))
        self.assertFalse(kconf.symbols["ARCH_I686"].get_value(kconf))
        self.assertTrue(kconf.symbols["USERLAND"].get_value(kconf))
        self.assertTrue(kconf.symbols["SMP"].get_value(kconf))

        out_path = os.path.join(self.temp_dir, ".config_test_64")
        kconf.write_config(out_path)
        self.assertTrue(os.path.exists(out_path))

        with open(out_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("CONFIG_ARCH_X86_64=y", content)
        self.assertIn("# CONFIG_ARCH_I686 is not set", content)

    def test_i686_defconfig_loading(self):
        kconf = Kconfig("Kconfig")
        kconf.load_config("arch/x86/configs/i686_defconfig")

        self.assertFalse(kconf.symbols["ARCH_X86_64"].get_value(kconf))
        self.assertTrue(kconf.symbols["ARCH_I686"].get_value(kconf))
        self.assertTrue(kconf.symbols["USERLAND"].get_value(kconf))

    def test_pure_defconfig_loading(self):
        kconf = Kconfig("Kconfig")
        kconf.load_config("arch/x86/configs/pure_defconfig")

        self.assertFalse(kconf.symbols["USERLAND"].get_value(kconf))
        self.assertFalse(kconf.symbols["USERLAND_COMPILER_KCC"].get_value(kconf))
        self.assertFalse(kconf.symbols["USERLAND_TESTS"].get_value(kconf))

    def test_autoconf_header_generation(self):
        kconf = Kconfig("Kconfig")
        kconf.load_config("arch/x86/configs/x86_64_defconfig")

        header_path = os.path.join(self.temp_dir, "autoconf.h")
        kconf.write_autoconf(header_path)
        self.assertTrue(os.path.exists(header_path))

        with open(header_path, "r", encoding="utf-8") as f:
            header_content = f.read()
        self.assertIn("#define CONFIG_ARCH_X86_64 1", header_content)
        self.assertIn('#define CONFIG_QEMU_MEM "128M"', header_content)
        self.assertIn("#define CONFIG_SMP 1", header_content)


class TestKbuildIncludePrimitives(unittest.TestCase):
    """Validates Kbuild compiler option probes and helper macros."""

    def test_kbuild_helpers_via_make(self):
        test_makefile = (
            "include scripts/Kbuild.include\n"
            "ASM := nasm\n"
            "CC := gcc\n"
            "test:\n"
            "\t@echo AS_OK:$(call as-option,-w+orphan-labels,-fail)\n"
            "\t@echo AS_BAD:$(call as-option,--invalid-option-xyz,-fail)\n"
            "\t@echo CC_OK:$(call cc-option,-O2,-fail)\n"
            "\t@echo CC_BAD:$(call cc-option,-finvalid-option-xyz,-fail)\n"
            "\t@echo CC_DIS_OK:$(call cc-disable-warning,unused-parameter)\n"
            "\t@echo CC_DIS_BAD:$(call cc-disable-warning,invalid-warning-xyz)\n"
            "\t@echo STRIP:$(call strip-slash,foo/bar/)\n"
            "\t@echo PREFIX:$(call add-prefix,arch/x86,boot.o kernel.o)\n"
        )
        res = subprocess.run(
            ["make", "-f", "-"],
            input=test_makefile,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True
        )
        out = res.stdout
        self.assertIn("AS_OK:-w+orphan-labels", out)
        self.assertIn("AS_BAD:-fail", out)
        self.assertIn("CC_OK:-O2", out)
        self.assertIn("CC_BAD:-fail", out)
        self.assertIn("CC_DIS_OK:-Wno-unused-parameter", out)
        self.assertIn("CC_DIS_BAD:", out)
        self.assertIn("STRIP:foo/bar", out)
        self.assertIn("PREFIX:arch/x86/boot.o arch/x86/kernel.o", out)


if __name__ == "__main__":
    unittest.main()
