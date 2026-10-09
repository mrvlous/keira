/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Standalone Shell (PID 2)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define SH_MAX_LINE 512
#define SH_MAX_ARGS 64
#define SH_PROMPT "sh-0.7$ "

static int last_exit_status = 0;

static int is_space(char c) {
    return c == ' ' || c == '\t' || c == '\r' || c == '\n';
}

static int tokenize_line(char *line, char **argv, int max_args) {
    int argc = 0;
    char *p = line;

    while (*p && argc < max_args - 1) {
        while (*p && is_space(*p)) {
            *p++ = '\0';
        }
        if (!*p) {
            break;
        }

        if (*p == '"' || *p == '\'') {
            char quote = *p++;
            argv[argc++] = p;
            while (*p && *p != quote) {
                p++;
            }
            if (*p) {
                *p++ = '\0';
            }
        } else {
            argv[argc++] = p;
            while (*p && !is_space(*p)) {
                p++;
            }
        }
    }
    argv[argc] = NULL;
    return argc;
}

static int builtin_cd(int argc, char **argv) {
    const char *target = (argc > 1) ? argv[1] : "/";
    if (chdir(target) != 0) {
        printf("sh: cd: %s: No such file or directory\n", target);
        return 1;
    }
    return 0;
}

static int builtin_pwd(void) {
    char cwd[256];
    if (getcwd(cwd, sizeof(cwd))) {
        printf("%s\n", cwd);
        return 0;
    }
    printf("sh: error retrieving current directory\n");
    return 1;
}

static int builtin_echo(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (arg[0] == '$') {
            if (strcmp(arg, "$?") == 0) {
                printf("%d", last_exit_status);
            } else {
                char *val = getenv(arg + 1);
                if (val) {
                    printf("%s", val);
                }
            }
        } else {
            printf("%s", arg);
        }
        if (i < argc - 1) {
            putchar(' ');
        }
    }
    putchar('\n');
    return 0;
}

static int builtin_export(int argc, char **argv) {
    if (argc < 2) {
        if (environ) {
            for (char **env = environ; *env; env++) {
                printf("declare -x %s\n", *env);
            }
        }
        return 0;
    }

    for (int i = 1; i < argc; i++) {
        char *eq = strchr(argv[i], '=');
        if (eq) {
            *eq = '\0';
            setenv(argv[i], eq + 1, 1);
            *eq = '=';
        } else {
            setenv(argv[i], "", 1);
        }
    }
    return 0;
}

static int builtin_help(void) {
    puts("Keira Standalone Userspace Shell (sh v0.7.0)");
    puts("Built-in Commands:");
    puts("  cd [dir]       Change current working directory");
    puts("  pwd            Print working directory");
    puts("  echo [args]    Display text or environment variables");
    puts("  export [K=V]   Set environment variable");
    puts("  clear          Clear console screen");
    puts("  help           Display this reference manual");
    puts("  exit [code]    Exit userspace shell");
    puts("Direct Execution:");
    puts("  Binaries in /bin (e.g. sysinfo, test_threads, kcc) execute via $PATH");
    return 0;
}

static int resolve_binary_path(const char *cmd, char *out_path, size_t out_size) {
    if (strchr(cmd, '/')) {
        snprintf(out_path, out_size, "%s", cmd);
        if (access(out_path, F_OK) == 0) {
            return 0;
        }
        snprintf(out_path, out_size, "%s.elf", cmd);
        if (access(out_path, F_OK) == 0) {
            return 0;
        }
        return -1;
    }

    snprintf(out_path, out_size, "/bin/%s", cmd);
    if (access(out_path, F_OK) == 0) {
        return 0;
    }

    snprintf(out_path, out_size, "/bin/%s.elf", cmd);
    if (access(out_path, F_OK) == 0) {
        return 0;
    }

    return -1;
}

static int execute_command(int argc, char **argv) {
    if (argc == 0 || !argv[0]) {
        return 0;
    }

    if (strcmp(argv[0], "cd") == 0) {
        return builtin_cd(argc, argv);
    }
    if (strcmp(argv[0], "pwd") == 0) {
        return builtin_pwd();
    }
    if (strcmp(argv[0], "echo") == 0) {
        return builtin_echo(argc, argv);
    }
    if (strcmp(argv[0], "export") == 0) {
        return builtin_export(argc, argv);
    }
    if (strcmp(argv[0], "clear") == 0) {
        printf("\x1b[2J\x1b[H");
        return 0;
    }
    if (strcmp(argv[0], "help") == 0) {
        return builtin_help();
    }
    if (strcmp(argv[0], "exit") == 0) {
        int code = (argc > 1) ? atoi(argv[1]) : 0;
        exit(code);
    }

    char resolved_path[256];
    if (resolve_binary_path(argv[0], resolved_path, sizeof(resolved_path)) != 0) {
        printf("sh: %s: command not found\n", argv[0]);
        return 127;
    }

    pid_t pid = fork();
    if (pid < 0) {
        printf("sh: fork failed\n");
        return 1;
    }

    if (pid == 0) {
        argv[0] = resolved_path;
        execve(resolved_path, argv, environ);
        printf("sh: execve %s failed\n", resolved_path);
        exit(127);
    }

    int status = 0;
    waitpid(pid, &status, 0);
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    return 1;
}

static int run_command_string(char *cmd_str) {
    char *argv[SH_MAX_ARGS];
    int argc = tokenize_line(cmd_str, argv, SH_MAX_ARGS);
    if (argc > 0) {
        last_exit_status = execute_command(argc, argv);
        return last_exit_status;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc >= 3 && strcmp(argv[1], "-c") == 0) {
        char cmd_buf[SH_MAX_LINE];
        cmd_buf[0] = '\0';
        size_t current_len = 0;
        for (int i = 2; i < argc; i++) {
            size_t arg_len = strlen(argv[i]);
            if (current_len + arg_len + 2 < sizeof(cmd_buf)) {
                if (current_len > 0) {
                    cmd_buf[current_len++] = ' ';
                    cmd_buf[current_len] = '\0';
                }
                memcpy(&cmd_buf[current_len], argv[i], arg_len);
                current_len += arg_len;
                cmd_buf[current_len] = '\0';
            }
        }
        return run_command_string(cmd_buf);
    }

    puts("Keira Standalone Userspace Shell (sh v0.7.0)");
    puts("Type 'help' for built-in commands or enter executable names.");

    char line_buf[SH_MAX_LINE];
    while (1) {
        printf(SH_PROMPT);
        fflush(stdout);

        if (!fgets(line_buf, sizeof(line_buf), stdin)) {
            putchar('\n');
            break;
        }

        size_t len = strlen(line_buf);
        while (len > 0 && (line_buf[len - 1] == '\n' || line_buf[len - 1] == '\r')) {
            line_buf[--len] = '\0';
        }

        if (len == 0) {
            continue;
        }

        run_command_string(line_buf);
    }

    return last_exit_status;
}
