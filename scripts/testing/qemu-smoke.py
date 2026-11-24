#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Run a headless Keira QEMU boot check and preserve its serial output."""

import argparse
import pathlib
import subprocess
import sys

REQUIRED_MARKERS = (
    "Keira Kernel v",
    "system initialization complete; spawning interactive shell",
)


def positive_float(value):
    """Parse a strictly positive timeout value for argparse."""
    try:
        timeout = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("timeout must be a number") from error

    if timeout <= 0:
        raise argparse.ArgumentTypeError("timeout must be greater than zero")
    return timeout


def missing_markers(output):
    """Return required boot markers that are absent from captured output."""
    return [marker for marker in REQUIRED_MARKERS if marker not in output]


def evaluate_result(output, timed_out, returncode):
    """Return whether QEMU reached the expected boot state before timeout."""
    if returncode not in (None, 0):
        return False, "QEMU exited with status {}".format(returncode)

    missing = missing_markers(output)
    if missing:
        return False, "missing boot marker(s): {}".format(", ".join(missing))

    if not timed_out:
        return False, "QEMU exited before the smoke-test timeout"

    return True, "kernel boot markers verified"


def _timeout_output(error):
    output = error.stdout or ""
    if isinstance(output, bytes):
        return output.decode("utf-8", errors="replace")
    return output


def run_smoke(command, timeout, log_path):
    """Execute QEMU, save its serial output and return a process exit status."""
    try:
        result = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            check=False,
        )
        output = result.stdout or ""
        timed_out = False
        returncode = result.returncode
    except subprocess.TimeoutExpired as error:
        output = _timeout_output(error)
        timed_out = True
        returncode = None
    except OSError as error:
        print("[FAIL] Unable to start QEMU: {}".format(error), file=sys.stderr)
        return 127

    try:
        path = pathlib.Path(log_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(output, encoding="utf-8")
    except OSError as error:
        print("[FAIL] Unable to write QEMU log: {}".format(error), file=sys.stderr)
        return 1

    passed, message = evaluate_result(output, timed_out, returncode)
    if not passed:
        print("[FAIL] QEMU smoke test: {}".format(message), file=sys.stderr)
        print("[INFO] Full QEMU output: {}".format(log_path), file=sys.stderr)
        return 1

    print("[PASS] QEMU smoke test: {}".format(message))
    print("[INFO] QEMU output saved to {}".format(log_path))
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=positive_float, default=10.0)
    parser.add_argument("--log", required=True, help="path for captured QEMU output")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)

    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("a QEMU command is required after --")

    return run_smoke(command, args.timeout, args.log)


if __name__ == "__main__":
    sys.exit(main())
