# SPDX-License-Identifier: GPL-2.0-only

"""Unit Tests for Kconfig Batch Processor and Autoconf Generation.

Validates loading baseline defconfig profiles (x86_64, i686, pure_defconfig),
writing .config and generating include/generated/autoconf.h headers.
"""

import os
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from kconfig import Kconfig

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

        out_path = os.path.join(self.temp_dir, ".config_test_32")
        kconf.write_config(out_path)
        self.assertTrue(os.path.exists(out_path))

        with open(out_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("CONFIG_ARCH_I686=y", content)
        self.assertIn("# CONFIG_ARCH_X86_64 is not set", content)

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

    def test_savedefconfig_spdx_header_and_string_quoting(self):
        cfg_path = os.path.join(self.temp_dir, ".config")
        def_path = os.path.join(self.temp_dir, "defconfig")

        kconf = Kconfig("Kconfig")
        kconf.load_config("arch/x86/configs/pure_defconfig")
        kconf.symbols["QEMU_MEM"].value = "256M"
        kconf.write_config(cfg_path)

        import subprocess

        res = subprocess.run(
            [
                sys.executable,
                "scripts/kconfig/conf.py",
                "-o",
                cfg_path,
                "--savedefconfig",
                def_path,
                "Kconfig",
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertIn("Saved minimal configuration", res.stdout)
        with open(def_path, "r", encoding="utf-8") as f:
            saved = f.read()
        self.assertTrue(saved.startswith("# SPDX-License-Identifier: GPL-2.0-only\n"))
        self.assertIn("# CONFIG_USERLAND is not set", saved)
        self.assertIn('CONFIG_QEMU_MEM="256M"', saved)

if __name__ == "__main__":
    unittest.main()
