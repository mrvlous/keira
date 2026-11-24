# SPDX-License-Identifier: GPL-2.0-only

"""Unit tests for the QEMU smoke-test result checks."""

import unittest

from qemu_smoke import REQUIRED_MARKERS, evaluate_result, missing_markers


class QemuSmokeTests(unittest.TestCase):
    def setUp(self):
        self.output = "\n".join(REQUIRED_MARKERS)

    def test_all_boot_markers_are_present(self):
        self.assertEqual(missing_markers(self.output), [])

    def test_missing_boot_marker_is_reported(self):
        missing = missing_markers(REQUIRED_MARKERS[0])
        self.assertEqual(missing, [REQUIRED_MARKERS[1]])

    def test_expected_timeout_after_boot_passes(self):
        passed, message = evaluate_result(self.output, True, None)
        self.assertTrue(passed)
        self.assertEqual(message, "kernel boot markers verified")

    def test_qemu_nonzero_exit_fails(self):
        passed, message = evaluate_result(self.output, False, 1)
        self.assertFalse(passed)
        self.assertEqual(message, "QEMU exited with status 1")

    def test_early_successful_exit_fails(self):
        passed, message = evaluate_result(self.output, False, 0)
        self.assertFalse(passed)
        self.assertEqual(message, "QEMU exited before the smoke-test timeout")

    def test_timeout_without_boot_markers_fails(self):
        passed, message = evaluate_result("bootloader output", True, None)
        self.assertFalse(passed)
        self.assertIn("missing boot marker(s)", message)


if __name__ == "__main__":
    unittest.main()
