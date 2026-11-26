# SPDX-License-Identifier: GPL-2.0-only

"""Unit Tests for Kconfig Grammar and Syntax Parsing.

Validates parsing the master root Kconfig manifest, recursive subsystem
manifests and boolean expression condition evaluation.
"""

import sys
import os
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from kconfig import Kconfig

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

    def test_or_condition_evaluation(self):
        self.kconf.symbols["SMP"].value = False
        self.kconf.symbols["USERLAND"].value = True
        self.assertTrue(self.kconf.eval_condition("SMP || USERLAND"))

        self.kconf.symbols["USERLAND"].value = False
        self.assertFalse(self.kconf.eval_condition("SMP || USERLAND"))

if __name__ == "__main__":
    unittest.main()
