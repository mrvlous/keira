# SPDX-License-Identifier: GPL-2.0-only
"""Unit tests for the QEMU smoke-test runner and result evaluator."""

import selectors
import unittest
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

from qemu_smoke import (
    evaluate_result,
    REQUIRED_MARKERS,
    run_qemu,
    SUPERVISOR_BOOT_MARKER,
)


VALID_OUTPUT = "\n".join(REQUIRED_MARKERS)

SUPERVISOR_OUTPUT = "\n".join(
    (
        REQUIRED_MARKERS[0],
        SUPERVISOR_BOOT_MARKER,
    )
)


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

    def test_successful_supervisor_boot(self):
        success, _ = evaluate_result(SUPERVISOR_OUTPUT, 0)
        self.assertTrue(success)

    def test_supervisor_boot_stopped_after_grace_period(self):
        success, _ = evaluate_result(
            SUPERVISOR_OUTPUT,
            -15,
            stopped_after_boot=True,
        )
        self.assertTrue(success)

    def test_missing_boot_markers(self):
        success, message = evaluate_result("", 0)
        self.assertFalse(success)
        self.assertIn("Missing boot marker", message)

    def test_missing_kernel_version_marker(self):
        output = REQUIRED_MARKERS[1]
        success, _ = evaluate_result(output, 0)
        self.assertFalse(success)

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


class TestRunQemu(unittest.TestCase):
    @patch("qemu_smoke.os.read")
    @patch("qemu_smoke.time.monotonic")
    @patch("qemu_smoke.selectors.DefaultSelector")
    @patch("qemu_smoke.subprocess.Popen")
    def test_grace_period_continues_after_boot_timeout(
        self,
        mock_popen,
        mock_selector,
        mock_monotonic,
        mock_read,
    ):
        stdout = MagicMock()
        key = SimpleNamespace(fileobj=stdout, fd=42)

        selector = MagicMock()
        selector.__enter__.return_value = selector
        selector.select.side_effect = [
            [(key, selectors.EVENT_READ)],
            [],
        ]
        selector.get_map.return_value = {42: key}
        mock_selector.return_value = selector

        process = MagicMock()
        process.stdout = stdout
        process.poll.return_value = None
        process.returncode = None
        process.terminate.side_effect = (
            lambda: setattr(process, "returncode", -15)
        )
        process.wait.return_value = -15
        mock_popen.return_value = process

        mock_read.return_value = VALID_OUTPUT.encode("utf-8")
        mock_monotonic.side_effect = [0.0, 0.9, 1.1, 3.2]

        result = run_qemu(
            ["qemu-system-x86_64"],
            timeout=1,
            grace_period=2,
            log_path=None,
        )

        self.assertEqual(result, 0)
        process.terminate.assert_called_once()
        process.wait.assert_called_once_with(timeout=3)
        mock_read.assert_called_once()


if __name__ == "__main__":
    unittest.main()
