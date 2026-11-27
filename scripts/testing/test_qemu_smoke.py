# SPDX-License-Identifier: GPL-2.0-only
"""Unit tests for the QEMU smoke-test result evaluator."""

import unittest

from qemu_smoke import evaluate_result, REQUIRED_MARKERS


VALID_OUTPUT = "\n".join(REQUIRED_MARKERS)


class TestEvaluateResult(unittest.TestCase):
    def test_successful_boot(self):
        success, _ = evaluate_result(VALID_OUTPUT, 0)
        self.assertTrue(success)

    def test_successful_boot_stopped_after_grace_period(self):
        success, _ = evaluate_result(
            VALID_OUTPUT,
            -15,
            stopped_after_boot=True,
        )
        self.assertTrue(success)

    def test_missing_boot_markers(self):
        success, message = evaluate_result("", 0)
        self.assertFalse(success)
        self.assertIn("Missing boot marker", message)

    def test_missing_one_boot_marker(self):
        success, _ = evaluate_result(REQUIRED_MARKERS[0], 0)
        self.assertFalse(success)

    def test_nonzero_exit_status(self):
        success, message = evaluate_result(VALID_OUTPUT, 1)
        self.assertFalse(success)
        self.assertIn("status 1", message)

    def test_zero_exit_without_boot_markers(self):
        success, _ = evaluate_result("", 0)
        self.assertFalse(success)

    def test_timeout_before_boot(self):
        success, message = evaluate_result(
            "",
            -9,
            timed_out=True,
        )
        self.assertFalse(success)
        self.assertIn("Missing boot marker", message)

    def test_timeout_after_boot_is_failure(self):
        success, message = evaluate_result(
            VALID_OUTPUT,
            -9,
            timed_out=True,
        )
        self.assertFalse(success)
        self.assertIn("timed out", message)

    def test_kernel_panic(self):
        output = VALID_OUTPUT + "\nKernel panic: test failure"
        success, message = evaluate_result(output, 0)
        self.assertFalse(success)
        self.assertIn("Kernel failure", message)

    def test_triple_fault(self):
        output = VALID_OUTPUT + "\nTriple fault detected"
        success, _ = evaluate_result(output, 0)
        self.assertFalse(success)

    def test_fatal_exception(self):
        output = VALID_OUTPUT + "\nFatal exception occurred"
        success, _ = evaluate_result(output, 0)
        self.assertFalse(success)

    def test_reboot_loop(self):
        output = VALID_OUTPUT + "\nKeira Kernel v0.8.0"
        success, message = evaluate_result(output, 0)
        self.assertFalse(success)
        self.assertIn("reboot loop", message)

    def test_panic_after_boot_is_not_ignored(self):
        output = VALID_OUTPUT + "\npanic: failure after initialization"
        success, _ = evaluate_result(
            output,
            -15,
            stopped_after_boot=True,
        )
        self.assertFalse(success)


if __name__ == "__main__":
    unittest.main()