/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "test_abi.h"

#include <stdio.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    puts("Keira Ring 3 Syscall Security & ABI Verification Harness");

    if (run_test_abi_proc() != 0) {
        puts("\n[FAIL] Process lifecycle & credential tests failed.");
        return 1;
    }

    if (run_test_abi_mem() != 0) {
        puts("\n[FAIL] Memory boundary, VMM & COW tests failed.");
        return 1;
    }

    if (run_test_abi_fs() != 0) {
        puts("\n[FAIL] Virtual filesystem & descriptor tests failed.");
        return 1;
    }

    if (run_test_abi_ipc() != 0) {
        puts("\n[FAIL] IPC, socket, termios & async engine tests failed.");
        return 1;
    }

    if (run_test_abi_signal() != 0) {
        puts("\n[FAIL] Signal handling & hardware fault containment tests failed.");
        return 1;
    }

    if (run_test_abi_stress() != 0) {
        puts("\n[FAIL] Multi-process stress & syscall boundary fuzzing failed.");
        return 1;
    }

    puts("\n[DONE] All Ring 3 Syscall Security & Fault Injection tests PASSED.");
    return 0;
}
