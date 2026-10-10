/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Standalone Shell (PID 2)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
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

        if (*p == '#') {
            *p = '\0';
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
    puts("  cd, go [dir]   Change current working directory");
    puts("  pwd            Print working directory");
    puts("  echo [args]    Display text or environment variables");
    puts("  export [K=V]   Set environment variable");
    puts("  clear          Clear console screen");
    puts("  help           Display this reference manual");
    puts("  exit [code]    Exit userspace shell");
    puts("Shell Features:");
    puts("  I/O Redir:     > file, >> file, < file");
    puts("  Pipelines:     cmd1 | cmd2");
    puts("  Chaining:      cmd1 && cmd2, cmd1 || cmd2, cmd1 ; cmd2");
    puts("Direct Execution:");
    puts("  Binaries in /bin (e.g. cat, ls, sysinfo, kcc) execute via $PATH");
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
        snprintf(out_path, out_size, "%s.sh", cmd);
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

    snprintf(out_path, out_size, "/bin/%s.sh", cmd);
    if (access(out_path, F_OK) == 0) {
        return 0;
    }

    return -1;
}

static int execute_single_command(int argc, char **argv) {
    if (argc == 0 || !argv[0]) {
        return 0;
    }

    char *out_file = NULL;
    int out_append = 0;
    char *in_file = NULL;
    char *clean_argv[SH_MAX_ARGS];
    int clean_argc = 0;

    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], ">") == 0) {
            if (i + 1 < argc) {
                out_file = argv[++i];
                out_append = 0;
            }
        } else if (strcmp(argv[i], ">>") == 0) {
            if (i + 1 < argc) {
                out_file = argv[++i];
                out_append = 1;
            }
        } else if (strcmp(argv[i], "<") == 0) {
            if (i + 1 < argc) {
                in_file = argv[++i];
            }
        } else if (strncmp(argv[i], ">>", 2) == 0 && argv[i][2] != '\0') {
            out_file = argv[i] + 2;
            out_append = 1;
        } else if (argv[i][0] == '>' && argv[i][1] != '\0') {
            out_file = argv[i] + 1;
            out_append = 0;
        } else if (argv[i][0] == '<' && argv[i][1] != '\0') {
            in_file = argv[i] + 1;
        } else {
            if (clean_argc < SH_MAX_ARGS - 1) {
                clean_argv[clean_argc++] = argv[i];
            }
        }
    }
    clean_argv[clean_argc] = NULL;

    if (clean_argc == 0) {
        return 0;
    }

    int redir_stdout = 0;
    int redir_stdin = 0;

    if (out_file) {
        int flags = O_WRONLY | O_CREAT | (out_append ? O_APPEND : O_TRUNC);
        int fd_out = open(out_file, flags, 0644);
        if (fd_out < 0) {
            printf("sh: cannot create %s\n", out_file);
            return 1;
        }
        dup2(fd_out, STDOUT_FILENO);
        close(fd_out);
        redir_stdout = 1;
    }

    if (in_file) {
        int fd_in = open(in_file, O_RDONLY, 0);
        if (fd_in < 0) {
            printf("sh: %s: No such file or directory\n", in_file);
            if (redir_stdout) {
                close(STDOUT_FILENO);
            }
            return 1;
        }
        dup2(fd_in, STDIN_FILENO);
        close(fd_in);
        redir_stdin = 1;
    }

    int ret_status = 0;

    if (strcmp(clean_argv[0], "cd") == 0 || strcmp(clean_argv[0], "go") == 0) {
        ret_status = builtin_cd(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "pwd") == 0) {
        ret_status = builtin_pwd();
    } else if (strcmp(clean_argv[0], "echo") == 0) {
        ret_status = builtin_echo(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "export") == 0) {
        ret_status = builtin_export(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "clear") == 0) {
        printf("\x1b[2J\x1b[H");
        ret_status = 0;
    } else if (strcmp(clean_argv[0], "help") == 0) {
        ret_status = builtin_help();
    } else if (strcmp(clean_argv[0], "exit") == 0) {
        int code = (clean_argc > 1) ? atoi(clean_argv[1]) : 0;
        exit(code);
    } else {
        char resolved_path[256];
        if (resolve_binary_path(clean_argv[0], resolved_path, sizeof(resolved_path)) != 0) {
            printf("sh: %s: command not found\n", clean_argv[0]);
            ret_status = 127;
        } else {
            clean_argv[0] = resolved_path;
            pid_t pid = execve(resolved_path, clean_argv, environ);
            if (pid < 0) {
                printf("sh: %s: execution failed\n", resolved_path);
                ret_status = 127;
            } else {
                int status = 0;
                waitpid(pid, &status, 0);
                if (WIFEXITED(status)) {
                    ret_status = WEXITSTATUS(status);
                } else {
                    ret_status = 0;
                }
            }
        }
    }

    fflush(stdout);
    fflush(stderr);

    if (redir_stdout) {
        close(STDOUT_FILENO);
    }
    if (redir_stdin) {
        close(STDIN_FILENO);
    }

    return ret_status;
}

static int execute_pipeline(char *cmd_line) {
    char *pipes[16];
    int num_pipes = 0;
    char *p = cmd_line;
    char *start = p;
    char in_quote = '\0';

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (*p == '|' && in_quote == '\0') {
            *p = '\0';
            pipes[num_pipes++] = start;
            start = p + 1;
            if (num_pipes >= 15) {
                break;
            }
        }
        p++;
    }
    pipes[num_pipes++] = start;

    if (num_pipes == 1) {
        char *argv[SH_MAX_ARGS];
        int argc = tokenize_line(pipes[0], argv, SH_MAX_ARGS);
        return execute_single_command(argc, argv);
    }

    int pipe_status = 0;

    for (int i = 0; i < num_pipes; i++) {
        char subcmd_buf[SH_MAX_LINE];
        if (i == 0) {
            snprintf(subcmd_buf, sizeof(subcmd_buf), "%s > /tmp/.sh_p0", pipes[0]);
            char *argv[SH_MAX_ARGS];
            int argc = tokenize_line(subcmd_buf, argv, SH_MAX_ARGS);
            pipe_status = execute_single_command(argc, argv);
        } else if (i == num_pipes - 1) {
            snprintf(subcmd_buf, sizeof(subcmd_buf), "%s < /tmp/.sh_p%d", pipes[i], i - 1);
            char *argv[SH_MAX_ARGS];
            int argc = tokenize_line(subcmd_buf, argv, SH_MAX_ARGS);
            pipe_status = execute_single_command(argc, argv);
        } else {
            snprintf(subcmd_buf, sizeof(subcmd_buf), "%s < /tmp/.sh_p%d > /tmp/.sh_p%d", pipes[i],
                     i - 1, i);
            char *argv[SH_MAX_ARGS];
            int argc = tokenize_line(subcmd_buf, argv, SH_MAX_ARGS);
            pipe_status = execute_single_command(argc, argv);
        }
    }

    for (int j = 0; j < num_pipes - 1; j++) {
        char fname[64];
        snprintf(fname, sizeof(fname), "/tmp/.sh_p%d", j);
        unlink(fname);
    }
    return pipe_status;
}

typedef enum { OP_NONE = 0, OP_AND, OP_OR } ChainOp;

typedef struct {
    char *cmd;
    ChainOp op;
} ChainNode;

static int execute_chained_statement(char *statement) {
    ChainNode nodes[16];
    int node_count = 0;

    char *p = statement;
    char *cmd_start = p;
    char in_quote = '\0';
    ChainOp next_op = OP_NONE;

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (in_quote == '\0') {
            if (*p == '&' && *(p + 1) == '&') {
                *p = '\0';
                nodes[node_count].cmd = cmd_start;
                nodes[node_count].op = next_op;
                node_count++;
                next_op = OP_AND;
                p++;
                cmd_start = p + 1;
            } else if (*p == '|' && *(p + 1) == '|') {
                *p = '\0';
                nodes[node_count].cmd = cmd_start;
                nodes[node_count].op = next_op;
                node_count++;
                next_op = OP_OR;
                p++;
                cmd_start = p + 1;
            }
        }
        p++;
    }

    nodes[node_count].cmd = cmd_start;
    nodes[node_count].op = next_op;
    node_count++;

    int status = 0;
    for (int i = 0; i < node_count; i++) {
        if (i > 0) {
            if (nodes[i].op == OP_AND && status != 0) {
                continue;
            }
            if (nodes[i].op == OP_OR && status == 0) {
                continue;
            }
        }

        while (*nodes[i].cmd && is_space(*nodes[i].cmd)) {
            nodes[i].cmd++;
        }
        if (*nodes[i].cmd == '\0') {
            continue;
        }

        status = execute_pipeline(nodes[i].cmd);
        last_exit_status = status;
    }

    return status;
}

static int run_command_string(char *cmd_str) {
    char *p = cmd_str;
    char *stmt_start = p;
    char in_quote = '\0';
    int status = 0;

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (*p == ';' && in_quote == '\0') {
            *p = '\0';
            while (*stmt_start && is_space(*stmt_start)) {
                stmt_start++;
            }
            if (*stmt_start) {
                status = execute_chained_statement(stmt_start);
            }
            stmt_start = p + 1;
        }
        p++;
    }

    while (*stmt_start && is_space(*stmt_start)) {
        stmt_start++;
    }
    if (*stmt_start) {
        status = execute_chained_statement(stmt_start);
    }

    return status;
}

static int run_script_file(const char *filename) {
    FILE *fp = fopen(filename, "r");
    if (!fp) {
        printf("sh: %s: No such file or directory\n", filename);
        return 127;
    }

    char line_buf[SH_MAX_LINE];
    int status = 0;
    while (fgets(line_buf, sizeof(line_buf), fp)) {
        size_t len = strlen(line_buf);
        while (len > 0 && (line_buf[len - 1] == '\n' || line_buf[len - 1] == '\r')) {
            line_buf[--len] = '\0';
        }
        if (len == 0) {
            continue;
        }
        status = run_command_string(line_buf);
    }

    fclose(fp);
    return status;
}

int main(int argc, char **argv) {
    if (argc >= 2 && (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0)) {
        puts("Usage: sh [-c command] [-h|--help] [script_file [args...]]");
        puts("");
        puts("Description:");
        puts("  Keira Canonical Standalone Userspace Shell (Ring 3).");
        puts("");
        puts("Options:");
        puts("  -c <cmd>       Execute command string non-interactively and exit");
        puts("  -h, --help     Show this help message and exit");
        puts("  script_file    Read and execute commands from script file");
        return 0;
    }

    if (argc >= 2 && strcmp(argv[1], "-c") == 0) {
        if (argc < 3) {
            printf("sh: -c: option requires an argument\n");
            return 2;
        }
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

    if (argc >= 2 && argv[1][0] != '-') {
        return run_script_file(argv[1]);
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
