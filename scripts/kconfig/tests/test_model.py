# SPDX-License-Identifier: GPL-2.0-only

"""Unit Tests for Kconfig Data Model Primitives.

Validates Symbol, Choice and Menu classes, property evaluation,
inheritance and visibility rules. Modeled after Linux kernel scripts/kconfig tests.
"""

import sys
import os
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
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

    def test_menu_visibility_inheritance(self):
        kconf = Kconfig.__new__(Kconfig)
        kconf.symbols = {}
        kconf.eval_condition = lambda cond: cond == "ACTIVE"

        parent_menu = Menu("Parent")
        parent_menu.depends_on.append("ACTIVE")
        child_menu = Menu("Child", parent=parent_menu)

        self.assertTrue(parent_menu.is_visible(kconf))
        self.assertTrue(child_menu.is_visible(kconf))

        parent_menu.depends_on = ["INACTIVE"]
        self.assertFalse(parent_menu.is_visible(kconf))
        self.assertFalse(child_menu.is_visible(kconf))


if __name__ == "__main__":
    unittest.main()
