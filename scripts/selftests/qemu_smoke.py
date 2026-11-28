#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Automated Headless QEMU Boot Smoke-Test Runner
# Copyright (C) 2026 Verdyan Arda

"""Automated Headless QEMU Boot Smoke-Test Runner for Keira Kernel.

Captures COM1 serial output from headless QEMU instances, validates userland
and pure Ring 0 boot completion markers and detects panics, faults or reboot
loops within bounded timeouts.
"""

import argparse
import os
from pathlib import Path
import re
import selectors
import subprocess
import sys
import time

REQUIRED_MARKERS = (
    "Keira Kernel v",
    "init: userspace execution interface ready",
)

SUPERVISOR_BOOT_MARKER = "pure Ring 0 kernel profile active"

FAILURE_PATTERNS = (
    re.compile(r"kernel panic", re.IGNORECASE),
    re.compile(r"\bpanic:", re.IGNORECASE),
    re.compile(r"triple fault", re.IGNORECASE),
    re.compile(r"fatal exception", re.IGNORECASE),
)

DEFAULT_TIMEOUT = 30
DEFAULT_GRACE_PERIOD = 2


def has_required_boot_markers(output):
    """Accept either userland init or pure Ring 0 boot completion markers."""
    return REQUIRED_MARKERS[0] in output and (
        REQUIRED_MARKERS[1] in output or SUPERVISOR_BOOT_MARKER in output
    )


def detect_failure(output):
    """Return failure reason if serial log contains a panic, fault or reboot loop."""
    for pattern in FAILURE_PATTERNS:
        if pattern.search(output):
            return f"Kernel failure detected: {pattern.pattern}"

    if output.count("Keira Kernel v") > 1:
        return "Kernel reboot loop detected"

    return None


def evaluate_result(
    output, returncode, timed_out=False, stopped_after_boot=False
):
    """Evaluate captured QEMU serial output and process termination state."""
    missing = []

    if REQUIRED_MARKERS[0] not in output:
        missing.append(REQUIRED_MARKERS[0])

    if (
        REQUIRED_MARKERS[1] not in output
        and SUPERVISOR_BOOT_MARKER not in output
    ):
        missing.append(f"{REQUIRED_MARKERS[1]} or {SUPERVISOR_BOOT_MARKER}")

    if missing:
        return False, "Missing boot marker(s): " + ", ".join(missing)

    failure_reason = detect_failure(output)
    if failure_reason is not None:
        return False, failure_reason

    if timed_out:
        return False, "QEMU timed out before successful completion"

    if stopped_after_boot:
        return True, "Boot markers verified; QEMU stopped after grace period"

    if returncode == 0:
        return True, "QEMU exited successfully after boot"

    if returncode is None:
        return False, "QEMU process did not exit"

    return False, f"QEMU exited with status {returncode}"


def run_qemu(command, timeout, grace_period, log_path):
    """Execute QEMU, capture serial output and validate the boot sequence."""
    if not command:
        print("[ERR] No QEMU command was provided.", file=sys.stderr)
        return 2

    output = bytearray()
    current_output = ""
    process = None
    timed_out = False
    stopped_after_boot = False
    boot_detected_at = None

    try:
        # Merge stderr into stdout so QEMU launch diagnostics and COM1 UART
        # logs are recorded chronologically in a single unbuffered stream.
        process = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            bufsize=0,
        )

        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            start_time = time.monotonic()

            while True:
                now = time.monotonic()

                if (
                    has_required_boot_markers(current_output)
                    and boot_detected_at is None
                ):
                    boot_detected_at = now
                    print("[INFO] Required boot markers detected.")

                if detect_failure(current_output) is not None:
                    break

                if (
                    boot_detected_at is not None
                    and now - boot_detected_at >= grace_period
                ):
                    stopped_after_boot = True
                    break

                # Enforce boot deadline only until markers appear; once booted,
                # let the post-boot grace window complete without false timeout.
                if boot_detected_at is None and now - start_time >= timeout:
                    timed_out = True
                    break

                new_data = False
                for key, _ in selector.select(timeout=0.1):
                    chunk = os.read(key.fd, 4096)
                    if chunk:
                        output.extend(chunk)
                        new_data = True
                    else:
                        selector.unregister(key.fileobj)

                if new_data:
                    current_output = output.decode("utf-8", errors="replace")

                if process.poll() is not None and not selector.get_map():
                    break

        if process.poll() is None:
            # Prefer SIGTERM first so QEMU flushes open disk and log descriptors.
            process.terminate()
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()

        returncode = process.returncode

    except OSError as error:
        print(f"[ERR] Failed to start QEMU: {error}", file=sys.stderr)
        return 2

    finally:
        if process is not None and process.stdout is not None:
            process.stdout.close()

    captured_output = output.decode("utf-8", errors="replace")

    if log_path:
        path = Path(log_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(captured_output, encoding="utf-8")

    success, message = evaluate_result(
        captured_output,
        returncode,
        timed_out=timed_out,
        stopped_after_boot=stopped_after_boot,
    )

    print(f"[{'DONE' if success else 'ERR'}] {message}")
    if not success:
        print(
            "[INFO] Full QEMU output is available in: "
            f"{log_path or 'stdout'}"
        )

    return 0 if success else 1


def main():
    """Parse CLI options and invoke the headless QEMU boot smoke runner."""
    parser = argparse.ArgumentParser(
        description="Automated Keira QEMU boot smoke test"
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_TIMEOUT,
        help="Maximum boot time in seconds",
    )
    parser.add_argument(
        "--grace-period",
        type=float,
        default=DEFAULT_GRACE_PERIOD,
        help="Time to monitor QEMU after successful boot markers",
    )
    parser.add_argument(
        "--log",
        help="Path to save captured QEMU output",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="QEMU command and its arguments",
    )

    args = parser.parse_args()
    command = args.command
    if command and command[0] == "--":
        command = command[1:]

    if args.timeout <= 0 or args.grace_period < 0:
        parser.error(
            "Timeout must be positive and grace period cannot be negative"
        )

    return run_qemu(
        command,
        args.timeout,
        args.grace_period,
        args.log,
    )


if __name__ == "__main__":
    sys.exit(main())
