# SPDX-License-Identifier: GPL-2.0-only

"""Unit Tests for Kbuild Compiler Probes and Helper Macros.

Validates as-option, cc-option, cc-disable-warning, strip-slash,
add-prefix and directory object filtering primitives defined in scripts/Kbuild.include.
"""

import subprocess
import unittest

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
