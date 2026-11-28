#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only

"""Master Test Runner for Keira Automated Selftest Harnesses.

Discovers and executes all unit tests under scripts/selftests/tests/ and
returns standard process exit codes for Makefile integration.
"""

import os
import sys
import unittest


def main():
    """Discover and execute all unit tests in the testing test suite."""
    tests_dir = os.path.dirname(os.path.abspath(__file__))
    testing_dir = os.path.dirname(tests_dir)
    sys.path.insert(0, testing_dir)

    loader = unittest.TestLoader()
    suite = loader.discover(start_dir=tests_dir, pattern="test_*.py")

    verbosity = 2 if ("-v" in sys.argv or "--verbose" in sys.argv) else 1
    runner = unittest.TextTestRunner(verbosity=verbosity)
    result = runner.run(suite)

    sys.exit(0 if result.wasSuccessful() else 1)


if __name__ == "__main__":
    main()
