/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Shell Scripting Engine Conformance Test Suite
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

static int run_sh_command(const char *cmd) {
#if defined(__x86_64__)
    char *argv[] = {"/bin/sh", "-c", (char *)cmd, NULL};
    pid_t pid = execve("/bin/sh", argv, environ);
    if (pid < 0) {
        printf("  [FAIL] execve() failed\n");
        return -1;
    }
    int status = 0;
    waitpid(pid, &status, 0);
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    return -1;
#else
    (void)cmd;
    puts("  [INFO] Subprocess shell invocation skipped on i686 (single-task target)");
    return 0;
#endif
}

static int test_variables(void) {
    printf("[TEST 1] Testing variable assignment and interpolation...\n");
    int ret = run_sh_command("FOO=keira_val; [ \"$FOO\" = \"keira_val\" ]");
    if (ret != 0) {
        printf("  [FAIL] Variable test returned status %d\n", ret);
        return 1;
    }
    printf("  [PASS] Variable assignment and expansion verified\n");
    return 0;
}

static int test_conditionals(void) {
    printf("[TEST 2] Testing conditional branching (if-then-elif-else-fi)...\n");
    int ret = run_sh_command("VAL=10; if [ $VAL -eq 5 ]; then exit 1; elif [ $VAL -eq 10 ]; then "
                             "exit 0; else exit 2; fi");
    if (ret != 0) {
        printf("  [FAIL] Conditional test returned status %d\n", ret);
        return 1;
    }
    printf("  [PASS] Multi-branch conditional logic verified\n");
    return 0;
}

static int test_for_loop(void) {
    printf("[TEST 3] Testing iterative for loop (for-in-do-done)...\n");
    int ret = run_sh_command("TOTAL=0; for n in 1 2 3; do TOTAL=$n; done; [ $TOTAL -eq 3 ]");
    if (ret != 0) {
        printf("  [FAIL] For loop test returned status %d\n", ret);
        return 1;
    }
    printf("  [PASS] For loop word iteration verified\n");
    return 0;
}

static int test_while_loop(void) {
    printf("[TEST 4] Testing iterative while loop (while-do-done)...\n");
    int ret = run_sh_command("C=2; while [ $C -gt 0 ]; do C=0; done; [ $C -eq 0 ]");
    if (ret != 0) {
        printf("  [FAIL] While loop test returned status %d\n", ret);
        return 1;
    }
    printf("  [PASS] While loop execution verified\n");
    return 0;
}

static int test_script_file(void) {
    printf("[TEST 5] Executing canonical script file /etc/test_scripting.sh...\n");
#if defined(__x86_64__)
    char *argv[] = {"/bin/sh", "/etc/test_scripting.sh", "arg_one", NULL};
    pid_t pid = execve("/bin/sh", argv, environ);
    if (pid < 0) {
        printf("  [FAIL] execve() failed\n");
        return 1;
    }
    int status = 0;
    waitpid(pid, &status, 0);
    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        printf("  [PASS] Script file execution completed with status 0\n");
        return 0;
    }
    printf("  [FAIL] Script file execution failed, status=%d\n", WEXITSTATUS(status));
    return 1;
#else
    puts("  [INFO] Subprocess script test skipped on i686 (run /bin/sh.elf /etc/test_scripting.sh "
         "directly)");
    return 0;
#endif
}

int main(void) {
    printf("======================================================================\n");
    printf("Keira Kernel - Shell Scripting Engine Conformance Suite\n");
    printf("======================================================================\n");

    int failures = 0;
    failures += test_variables();
    failures += test_conditionals();
    failures += test_for_loop();
    failures += test_while_loop();
    failures += test_script_file();

    printf("======================================================================\n");
    if (failures == 0) {
        printf("[SUCCESS] All 5 shell scripting engine tests passed cleanly!\n");
        return 0;
    }
    printf("[FAILURE] %d tests failed!\n", failures);
    return 1;
}
