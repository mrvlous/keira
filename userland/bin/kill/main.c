/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Process Signal Utility (kill)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void print_usage(void) {
    puts("Usage: kill [-<sig>] <pid>...");
    puts("       kill -l");
    puts("");
    puts("Description:");
    puts("  Send a POSIX signal to specified process IDs.");
    puts("");
    puts("Options:");
    puts("  -<sig>      Numeric signal number (default: 15 for SIGTERM)");
    puts("  -l, --list  List all supported signal names and values");
    puts("  -h, --help  Display this help reference and exit");
}

static void list_signals(void) {
    puts(" 1) SIGHUP       2) SIGINT       3) SIGQUIT      6) SIGABRT");
    puts(" 9) SIGKILL     11) SIGSEGV     13) SIGPIPE     14) SIGALRM");
    puts("15) SIGTERM     17) SIGCHLD     18) SIGCONT     19) SIGSTOP");
}

int main(int argc, char **argv) {
    if (argc < 2) {
        print_usage();
        return 1;
    }

    int sig = SIGTERM;
    int target_pid = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-l") == 0 || strcmp(argv[i], "--list") == 0) {
            list_signals();
            return 0;
        }
        if (argv[i][0] == '-') {
            sig = atoi(argv[i] + 1);
            if (sig <= 0 || sig > 32) {
                fprintf(stderr, "kill: invalid signal number '%s'\n", argv[i]);
                return 1;
            }
        } else {
            target_pid = atoi(argv[i]);
            if (target_pid < 0) {
                fprintf(stderr, "kill: invalid process id '%s'\n", argv[i]);
                return 1;
            }

            if (kill((pid_t)target_pid, sig) < 0) {
                fprintf(stderr, "kill: failed to deliver signal %d to pid %d\n", sig, target_pid);
                return 1;
            }
        }
    }

    if (target_pid < 0) {
        fprintf(stderr, "kill: missing process id operand\n");
        return 1;
    }

    return 0;
}
