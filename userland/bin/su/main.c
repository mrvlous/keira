/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Switch User (su) Utility
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

static void print_usage(void) {
    puts("Usage: su [OPTIONS] [-] [UID [ARG]...]");
    puts("");
    puts("Description:");
    puts("  Switch the process user ID and group ID in the kernel credentials engine.");
    puts("  If UID is omitted, UID 0 (root) is assumed.");
    puts("");
    puts("Options:");
    puts("  -c, --command COMMAND  Pass COMMAND to the invoked shell");
    puts("  -s, --shell SHELL      Run specified SHELL (default: /bin/sh)");
    puts("  -h, --help             Display this help reference and exit");
    puts("  -v, --version          Display version information and exit");
}

int main(int argc, char **argv) {
    const char *command = NULL;
    const char *custom_shell = "/bin/sh";
    uid_t target_uid = 0;
    gid_t target_gid = 0;
    int arg_idx = 1;

    while (arg_idx < argc) {
        if (strcmp(argv[arg_idx], "-h") == 0 || strcmp(argv[arg_idx], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[arg_idx], "-v") == 0 || strcmp(argv[arg_idx], "--version") == 0) {
            puts("su (Keira Coreutils) 0.7.0");
            return 0;
        }
        if (strcmp(argv[arg_idx], "-") == 0 || strcmp(argv[arg_idx], "-l") == 0 ||
            strcmp(argv[arg_idx], "--login") == 0) {
            arg_idx++;
            continue;
        }
        if (strcmp(argv[arg_idx], "-c") == 0 || strcmp(argv[arg_idx], "--command") == 0) {
            if (arg_idx + 1 >= argc) {
                fprintf(stderr, "su: option '%s' requires an argument\n", argv[arg_idx]);
                return 1;
            }
            command = argv[arg_idx + 1];
            arg_idx += 2;
            continue;
        }
        if (strcmp(argv[arg_idx], "-s") == 0 || strcmp(argv[arg_idx], "--shell") == 0) {
            if (arg_idx + 1 >= argc) {
                fprintf(stderr, "su: option '%s' requires an argument\n", argv[arg_idx]);
                return 1;
            }
            custom_shell = argv[arg_idx + 1];
            arg_idx += 2;
            continue;
        }
        if (argv[arg_idx][0] == '-') {
            fprintf(stderr, "su: invalid option -- '%s'\n", argv[arg_idx]);
            fprintf(stderr, "Try 'su --help' for more information.\n");
            return 1;
        }

        const char *arg = argv[arg_idx++];
        if (strcmp(arg, "root") == 0) {
            target_uid = 0;
            target_gid = 0;
        } else {
            target_uid = (uid_t)atoi(arg);
            target_gid = (gid_t)target_uid;
        }
        break;
    }

    /* Set GID in kernel */
    if (setgid(target_gid) != 0) {
        fprintf(stderr, "su: cannot set GID %u: Operation not permitted\n",
                (unsigned int)target_gid);
        return 1;
    }

    /* Set UID in kernel */
    if (setuid(target_uid) != 0) {
        fprintf(stderr, "su: cannot set UID %u: Operation not permitted\n",
                (unsigned int)target_uid);
        return 1;
    }

    /* Update userland environment */
    if (target_uid == 0) {
        setenv("USER", "root", 1);
        setenv("LOGNAME", "root", 1);
    } else {
        char uid_str[16];
        snprintf(uid_str, sizeof(uid_str), "%u", (unsigned int)target_uid);
        setenv("USER", uid_str, 1);
        setenv("LOGNAME", uid_str, 1);
    }
    setenv("SHELL", custom_shell, 1);

    /* Launch shell */
    pid_t child_pid = -1;
    if (command) {
        char *exec_args[4];
        exec_args[0] = (char *)custom_shell;
        exec_args[1] = "-c";
        exec_args[2] = (char *)command;
        exec_args[3] = NULL;
        child_pid = execve(custom_shell, exec_args, environ);
    } else {
        char *exec_args[2];
        exec_args[0] = (char *)custom_shell;
        exec_args[1] = NULL;
        child_pid = execve(custom_shell, exec_args, environ);
    }

    if (child_pid > 0) {
        int status = 0;
        waitpid(child_pid, &status, 0);
        return status;
    }

    fprintf(stderr, "su: execve '%s' failed\n", custom_shell);
    return 1;
}
