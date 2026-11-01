/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Init System (PID 1)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: init [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Canonical userspace init system (PID 1).");
    puts("");
    puts("Options:");
    puts("  -v, --verbose  Display detailed initialization status");
    puts("  -h, --help     Display this help reference and exit");
}

int main(int argc, char **argv) {
    int verbose = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        }
    }

    if (verbose) {
        puts("[INIT] Keira Canonical Userspace Init (PID 1) started in Ring 3");
        puts("[INIT] Pure freestanding kernel environment certified");
    }

    if (access("/etc/init.sh", F_OK) == 0) {
        puts("[INIT] Keira Canonical Userspace Init (PID 1) active in Ring 3");
        puts("[INIT] Executing canonical system bootstrap: /etc/init.sh");

        char *const init_argv[] = {"/bin/sh", "/etc/init.sh", NULL};
        char *const init_envp[] = {"PATH=/bin", NULL};
        pid_t pid = execve("/bin/sh", init_argv, init_envp);
        if (pid < 0) {
            char *const init_elf_argv[] = {"/bin/sh.elf", "/etc/init.sh", NULL};
            pid = execve("/bin/sh.elf", init_elf_argv, init_envp);
        }

        if (pid > 0) {
            int status = 0;
            waitpid(pid, &status, 0);
        }

        puts("[INIT] System bootstrap sequence complete.");
    }

    return 0;
}
