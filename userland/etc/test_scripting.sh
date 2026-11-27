#!/bin/sh
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.

# Standalone Shell Scripting Engine Test Suite
#
# Validates variable expansion, positional arguments, conditional execution,
# iterative loops and builtin test expressions in Ring 3 userspace.

echo "[TEST] Starting Keira POSIX shell scripting verification suite..."

# 1. Variable Assignment & Interpolation
NAME=keira
VERSION=0.9.0
echo "[TEST 1] Variables: name=$NAME version=$VERSION pid=$$"

# 2. Positional Parameters & Argument Count
echo "[TEST 2] Positional args: script=$0 arg1=$1 count=$#"

# 3. Builtin test / [ Evaluator
echo "[TEST 3] Evaluating file attribute and string test conditions..."
if [ -f /etc/hostname ]; then
    echo "  [PASS] /etc/hostname exists and is regular file"
else
    echo "  [FAIL] /etc/hostname missing"
    exit 1
fi

if [ -d /bin ]; then
    echo "  [PASS] /bin directory verified"
else
    echo "  [FAIL] /bin directory missing"
    exit 1
fi

if [ "$NAME" = "keira" ]; then
    echo "  [PASS] String equality test matched"
else
    echo "  [FAIL] String equality failed"
    exit 1
fi

if [ 42 -gt 10 ]; then
    echo "  [PASS] Integer comparison test matched"
else
    echo "  [FAIL] Integer comparison failed"
    exit 1
fi

# 4. Multi-Branch Conditional (if-elif-else-fi)
STATUS_CODE=200
if [ $STATUS_CODE -eq 404 ]; then
    echo "  [FAIL] Unexpected 404 status"
    exit 1
elif [ $STATUS_CODE -eq 200 ]; then
    echo "  [PASS] Status code 200 matched in elif branch"
else
    echo "  [FAIL] Else branch erroneously hit"
    exit 1
fi

# 5. Iterative For Loop (for-in-do-done)
echo "[TEST 4] Executing for loop over word list..."
for item in alpha beta gamma delta; do
    echo "  [ITEM] $item"
done

# 6. Iterative While Loop (while-do-done)
echo "[TEST 5] Executing while loop countdown..."
COUNT=3
while [ $COUNT -gt 0 ]; do
    echo "  [TICK] countdown=$COUNT"
    if [ $COUNT -eq 3 ]; then
        COUNT=2
    elif [ $COUNT -eq 2 ]; then
        COUNT=1
    else
        COUNT=0
    fi
done

# 7. Exit Code Propagation ($?)
test -f /etc/hostname
LAST_ERR=$?
echo "[TEST 6] Exit status test: last_status=$LAST_ERR"

echo "[TEST] All shell scripting language tests passed successfully!"
exit 0
