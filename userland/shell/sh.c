// SPDX-License-Identifier: GPL-2.0-only
/*
 * Keira Kernel - Freestanding Canonical Standalone Shell (PID 2)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
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

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/version.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>

#define SH_MAX_LINE 512
#define SH_MAX_ARGS 64
#define SH_HISTORY_MAX 32
#define SH_MAX_VARS 128
#define SH_VAR_NAME_MAX 64
#define SH_VAR_VAL_MAX 256

typedef struct {
    char name[SH_VAR_NAME_MAX];
    char value[SH_VAR_VAL_MAX];
    int exported;
} sh_var_t;

static sh_var_t sh_vars[SH_MAX_VARS];
static int sh_var_count = 0;

static int sh_argc = 0;
static char sh_argv_storage[SH_MAX_ARGS][SH_MAX_LINE];
static char *sh_argv[SH_MAX_ARGS];

static int sh_loop_break = 0;
static int sh_loop_continue = 0;
static int sh_return_flag = 0;

static int run_command_string(char *cmd_str);
static int execute_top_statement(char *stmt);
static int run_script_stream(FILE *fp);

static char sh_history[SH_HISTORY_MAX][SH_MAX_LINE];
static int sh_history_count = 0;

static void sh_history_push(const char *cmd) {
    if (!cmd || cmd[0] == '\0') {
        return;
    }
    if (sh_history_count > 0) {
        int last_idx = (sh_history_count - 1) % SH_HISTORY_MAX;
        if (strcmp(sh_history[last_idx], cmd) == 0) {
            return;
        }
    }
    int idx = sh_history_count % SH_HISTORY_MAX;
    strncpy(sh_history[idx], cmd, SH_MAX_LINE - 1);
    sh_history[idx][SH_MAX_LINE - 1] = '\0';
    sh_history_count++;
}

static int builtin_history(void) {
    int start = (sh_history_count > SH_HISTORY_MAX) ? (sh_history_count - SH_HISTORY_MAX) : 0;
    for (int i = start; i < sh_history_count; i++) {
        int idx = i % SH_HISTORY_MAX;
        printf(" %4d  %s\n", i + 1, sh_history[idx]);
    }
    return 0;
}

static void print_shell_prompt(void) {
    char hostname[64] = "keira";
    int fd = open("/etc/hostname", O_RDONLY, 0);
    if (fd >= 0) {
        ssize_t n = read(fd, hostname, sizeof(hostname) - 1);
        if (n > 0) {
            while (n > 0 &&
                   (hostname[n - 1] == '\n' || hostname[n - 1] == '\r' || hostname[n - 1] == ' ')) {
                n--;
            }
            hostname[n] = '\0';
        }
        close(fd);
    }
    char cwd[256] = "/";
    if (!getcwd(cwd, sizeof(cwd))) {
        strcpy(cwd, "/");
    }
    uid_t euid = geteuid();
    char prompt_char = (euid == 0) ? '#' : '$';
    printf("%s:%s%c ", hostname, cwd, prompt_char);
    fflush(stdout);
}

static int last_exit_status = 0;

#define SH_MAX_JOBS 32

typedef enum { JOB_EMPTY = 0, JOB_RUNNING, JOB_STOPPED, JOB_DONE } job_state_t;

typedef struct {
    int id;
    pid_t pid;
    pid_t pgid;
    job_state_t state;
    int exit_status;
    char cmd[SH_MAX_LINE];
} job_t;

static job_t job_table[SH_MAX_JOBS];

static job_t *job_find_by_id(int id) {
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state != JOB_EMPTY && job_table[i].id == id) {
            return &job_table[i];
        }
    }
    return NULL;
}

static job_t *job_get_latest(void) {
    job_t *latest = NULL;
    int max_id = -1;
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state != JOB_EMPTY && job_table[i].id > max_id) {
            max_id = job_table[i].id;
            latest = &job_table[i];
        }
    }
    return latest;
}

static job_t *job_add(pid_t pid, pid_t pgid, const char *cmd) {
    int next_id = 1;
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state != JOB_EMPTY && job_table[i].id >= next_id) {
            next_id = job_table[i].id + 1;
        }
    }

    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state == JOB_EMPTY) {
            job_table[i].id = next_id;
            job_table[i].pid = pid;
            job_table[i].pgid = pgid;
            job_table[i].state = JOB_RUNNING;
            job_table[i].exit_status = 0;
            strncpy(job_table[i].cmd, cmd, sizeof(job_table[i].cmd) - 1);
            job_table[i].cmd[sizeof(job_table[i].cmd) - 1] = '\0';
            return &job_table[i];
        }
    }
    return NULL;
}

static void job_reap(void) {
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state == JOB_RUNNING) {
            int status = 0;
            pid_t ret = waitpid(job_table[i].pid, &status, WNOHANG);
            if (ret == job_table[i].pid) {
                job_table[i].state = JOB_DONE;
                job_table[i].exit_status = WIFEXITED(status) ? WEXITSTATUS(status) : 0;
                printf("[%d]+  Done                    %s\n", job_table[i].id, job_table[i].cmd);
                job_table[i].state = JOB_EMPTY;
            } else if (ret < 0 && errno == ECHILD) {
                job_table[i].state = JOB_DONE;
                printf("[%d]+  Done                    %s\n", job_table[i].id, job_table[i].cmd);
                job_table[i].state = JOB_EMPTY;
            }
        }
    }
    fflush(stdout);
}

static int builtin_jobs(int argc, char **argv) {
    (void)argc;
    (void)argv;
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state == JOB_RUNNING) {
            int status = 0;
            pid_t ret = waitpid(job_table[i].pid, &status, WNOHANG);
            if (ret == job_table[i].pid) {
                job_table[i].state = JOB_DONE;
                job_table[i].exit_status = WIFEXITED(status) ? WEXITSTATUS(status) : 0;
            } else if (ret < 0 && errno == ECHILD) {
                job_table[i].state = JOB_DONE;
            }
        }
    }

    job_t *latest = job_get_latest();
    for (int i = 0; i < SH_MAX_JOBS; i++) {
        if (job_table[i].state != JOB_EMPTY) {
            char marker = (&job_table[i] == latest) ? '+' : '-';
            const char *state_str = (job_table[i].state == JOB_RUNNING)   ? "Running"
                                    : (job_table[i].state == JOB_STOPPED) ? "Stopped"
                                                                          : "Done";
            printf("[%d]%c  %-24s%s\n", job_table[i].id, marker, state_str, job_table[i].cmd);
            if (job_table[i].state == JOB_DONE) {
                job_table[i].state = JOB_EMPTY;
            }
        }
    }
    fflush(stdout);
    return 0;
}

static int builtin_fg(int argc, char **argv) {
    job_t *job = NULL;
    if (argc > 1) {
        const char *arg = argv[1];
        if (arg[0] == '%') {
            arg++;
        }
        int id = atoi(arg);
        job = job_find_by_id(id);
        if (!job) {
            printf("sh: fg: %s: no such job\n", argv[1]);
            return 1;
        }
    } else {
        job = job_get_latest();
        if (!job) {
            printf("sh: fg: current: no such job\n");
            return 1;
        }
    }

    printf("%s\n", job->cmd);
    fflush(stdout);

    int status = 0;
    pid_t target_pid = job->pid;
    job->state = JOB_EMPTY;

    pid_t ret = waitpid(target_pid, &status, 0);
    if (ret > 0 && WIFEXITED(status)) {
        last_exit_status = WEXITSTATUS(status);
        return last_exit_status;
    }
    return 0;
}

static int builtin_bg(int argc, char **argv) {
    job_t *job = NULL;
    if (argc > 1) {
        const char *arg = argv[1];
        if (arg[0] == '%') {
            arg++;
        }
        int id = atoi(arg);
        job = job_find_by_id(id);
        if (!job) {
            printf("sh: bg: %s: no such job\n", argv[1]);
            return 1;
        }
    } else {
        job = job_get_latest();
        if (!job) {
            printf("sh: bg: current: no such job\n");
            return 1;
        }
    }

    job->state = JOB_RUNNING;
    printf("[%d]+ %s\n", job->id, job->cmd);
    fflush(stdout);
    return 0;
}

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
        if (errno == 20) {
            printf("sh: cd: %s: Not a directory\n", target);
        } else {
            printf("sh: cd: %s: No such file or directory\n", target);
        }
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

static sh_var_t *sh_var_find(const char *name) {
    if (!name || name[0] == '\0') {
        return NULL;
    }
    for (int i = 0; i < sh_var_count; i++) {
        if (strcmp(sh_vars[i].name, name) == 0) {
            return &sh_vars[i];
        }
    }
    return NULL;
}

static const char *sh_var_get(const char *name) {
    if (!name || name[0] == '\0') {
        return NULL;
    }
    sh_var_t *v = sh_var_find(name);
    if (v) {
        return v->value;
    }
    return getenv(name);
}

static int sh_var_set(const char *name, const char *value, int is_export) {
    if (!name || name[0] == '\0') {
        return -1;
    }
    sh_var_t *v = sh_var_find(name);
    if (v) {
        strncpy(v->value, value ? value : "", SH_VAR_VAL_MAX - 1);
        v->value[SH_VAR_VAL_MAX - 1] = '\0';
        if (is_export) {
            v->exported = 1;
        }
        if (v->exported) {
            setenv(name, v->value, 1);
        }
        return 0;
    }
    if (sh_var_count < SH_MAX_VARS) {
        strncpy(sh_vars[sh_var_count].name, name, SH_VAR_NAME_MAX - 1);
        sh_vars[sh_var_count].name[SH_VAR_NAME_MAX - 1] = '\0';
        strncpy(sh_vars[sh_var_count].value, value ? value : "", SH_VAR_VAL_MAX - 1);
        sh_vars[sh_var_count].value[SH_VAR_VAL_MAX - 1] = '\0';
        sh_vars[sh_var_count].exported = is_export;
        if (is_export) {
            setenv(name, value ? value : "", 1);
        }
        sh_var_count++;
        return 0;
    }
    return -1;
}

static int sh_var_unset(const char *name) {
    if (!name || name[0] == '\0') {
        return -1;
    }
    for (int i = 0; i < sh_var_count; i++) {
        if (strcmp(sh_vars[i].name, name) == 0) {
            for (int j = i; j < sh_var_count - 1; j++) {
                sh_vars[j] = sh_vars[j + 1];
            }
            sh_var_count--;
            unsetenv(name);
            return 0;
        }
    }
    unsetenv(name);
    return 0;
}

static void expand_parameters(const char *src, char *dst, size_t dst_size) {
    size_t d = 0;
    const char *s = src;
    int in_squote = 0;
    int in_dquote = 0;

    while (*s && d + 1 < dst_size) {
        if (*s == '\'' && !in_dquote) {
            in_squote = !in_squote;
            dst[d++] = *s++;
        } else if (*s == '"' && !in_squote) {
            in_dquote = !in_dquote;
            dst[d++] = *s++;
        } else if (*s == '$' && !in_squote) {
            s++;
            if (*s == '?') {
                char num[16];
                snprintf(num, sizeof(num), "%d", last_exit_status);
                size_t nlen = strlen(num);
                if (d + nlen < dst_size) {
                    memcpy(&dst[d], num, nlen);
                    d += nlen;
                }
                s++;
            } else if (*s == '$') {
                char num[16];
                snprintf(num, sizeof(num), "%d", (int)getpid());
                size_t nlen = strlen(num);
                if (d + nlen < dst_size) {
                    memcpy(&dst[d], num, nlen);
                    d += nlen;
                }
                s++;
            } else if (*s == '#') {
                char num[16];
                snprintf(num, sizeof(num), "%d", sh_argc > 1 ? (sh_argc - 1) : 0);
                size_t nlen = strlen(num);
                if (d + nlen < dst_size) {
                    memcpy(&dst[d], num, nlen);
                    d += nlen;
                }
                s++;
            } else if (*s >= '0' && *s <= '9') {
                int arg_idx = *s - '0';
                s++;
                const char *val = "";
                if (arg_idx < sh_argc) {
                    val = sh_argv[arg_idx];
                }
                size_t vlen = strlen(val);
                if (d + vlen < dst_size) {
                    memcpy(&dst[d], val, vlen);
                    d += vlen;
                }
            } else if (*s == '*' || *s == '@') {
                s++;
                for (int i = 1; i < sh_argc; i++) {
                    if (i > 1 && d + 1 < dst_size) {
                        dst[d++] = ' ';
                    }
                    size_t vlen = strlen(sh_argv[i]);
                    if (d + vlen < dst_size) {
                        memcpy(&dst[d], sh_argv[i], vlen);
                        d += vlen;
                    }
                }
            } else if (*s == '{') {
                s++;
                char var_name[64];
                size_t v_idx = 0;
                while (*s && *s != '}' && v_idx + 1 < sizeof(var_name)) {
                    var_name[v_idx++] = *s++;
                }
                var_name[v_idx] = '\0';
                if (*s == '}') {
                    s++;
                }
                const char *val = sh_var_get(var_name);
                if (val) {
                    size_t vlen = strlen(val);
                    if (d + vlen < dst_size) {
                        memcpy(&dst[d], val, vlen);
                        d += vlen;
                    }
                }
            } else if ((*s >= 'a' && *s <= 'z') || (*s >= 'A' && *s <= 'Z') || *s == '_') {
                char var_name[64];
                size_t v_idx = 0;
                while ((*s >= 'a' && *s <= 'z') || (*s >= 'A' && *s <= 'Z') ||
                       (*s >= '0' && *s <= '9') || *s == '_') {
                    if (v_idx + 1 < sizeof(var_name)) {
                        var_name[v_idx++] = *s;
                    }
                    s++;
                }
                var_name[v_idx] = '\0';
                const char *val = sh_var_get(var_name);
                if (val) {
                    size_t vlen = strlen(val);
                    if (d + vlen < dst_size) {
                        memcpy(&dst[d], val, vlen);
                        d += vlen;
                    }
                }
            } else {
                dst[d++] = '$';
            }
        } else {
            dst[d++] = *s++;
        }
    }
    dst[d] = '\0';
}

static int eval_test(int argc, char **argv) {
    if (argc <= 0) {
        return 1;
    }

    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], "-o") == 0 && i > 0 && i < argc - 1) {
            int left = eval_test(i, argv);
            if (left == 0) {
                return 0;
            }
            return eval_test(argc - i - 1, argv + i + 1);
        }
    }

    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], "-a") == 0 && i > 0 && i < argc - 1) {
            int left = eval_test(i, argv);
            if (left != 0) {
                return 1;
            }
            return eval_test(argc - i - 1, argv + i + 1);
        }
    }

    if (strcmp(argv[0], "!") == 0) {
        return eval_test(argc - 1, argv + 1) == 0 ? 1 : 0;
    }

    if (argc >= 2 && strcmp(argv[0], "(") == 0 && strcmp(argv[argc - 1], ")") == 0) {
        return eval_test(argc - 2, argv + 1);
    }

    if (argc == 1) {
        return argv[0][0] != '\0' ? 0 : 1;
    }

    if (argc == 2) {
        const char *op = argv[0];
        const char *target = argv[1];

        if (strcmp(op, "-z") == 0) {
            return strlen(target) == 0 ? 0 : 1;
        }
        if (strcmp(op, "-n") == 0) {
            return strlen(target) > 0 ? 0 : 1;
        }

        struct stat st;
        if (strcmp(op, "-e") == 0) {
            return stat(target, &st) == 0 ? 0 : 1;
        }
        if (strcmp(op, "-f") == 0) {
            return (stat(target, &st) == 0 && S_ISREG(st.st_mode)) ? 0 : 1;
        }
        if (strcmp(op, "-d") == 0) {
            return (stat(target, &st) == 0 && S_ISDIR(st.st_mode)) ? 0 : 1;
        }
        if (strcmp(op, "-s") == 0) {
            return (stat(target, &st) == 0 && st.st_size > 0) ? 0 : 1;
        }
        if (strcmp(op, "-r") == 0) {
            return access(target, R_OK) == 0 ? 0 : 1;
        }
        if (strcmp(op, "-w") == 0) {
            return access(target, W_OK) == 0 ? 0 : 1;
        }
        if (strcmp(op, "-x") == 0) {
            return access(target, X_OK) == 0 ? 0 : 1;
        }

        return 1;
    }

    if (argc == 3) {
        const char *left = argv[0];
        const char *op = argv[1];
        const char *right = argv[2];

        if (strcmp(op, "=") == 0 || strcmp(op, "==") == 0) {
            return strcmp(left, right) == 0 ? 0 : 1;
        }
        if (strcmp(op, "!=") == 0) {
            return strcmp(left, right) != 0 ? 0 : 1;
        }

        int n1 = atoi(left);
        int n2 = atoi(right);

        if (strcmp(op, "-eq") == 0) {
            return n1 == n2 ? 0 : 1;
        }
        if (strcmp(op, "-ne") == 0) {
            return n1 != n2 ? 0 : 1;
        }
        if (strcmp(op, "-lt") == 0) {
            return n1 < n2 ? 0 : 1;
        }
        if (strcmp(op, "-le") == 0) {
            return n1 <= n2 ? 0 : 1;
        }
        if (strcmp(op, "-gt") == 0) {
            return n1 > n2 ? 0 : 1;
        }
        if (strcmp(op, "-ge") == 0) {
            return n1 >= n2 ? 0 : 1;
        }

        return 1;
    }

    return 1;
}

static int builtin_test(int argc, char **argv) {
    if (argc > 0 && (strcmp(argv[0], "[") == 0 || strcmp(argv[0], "[.elf") == 0)) {
        if (argc < 2 || strcmp(argv[argc - 1], "]") != 0) {
            fprintf(stderr, "sh: [: missing `]'\n");
            return 2;
        }
        argc--;
    }
    return eval_test(argc - 1, argv + 1);
}

static int builtin_echo(int argc, char **argv) {
    int newline = 1;
    int start = 1;
    if (argc > 1 && strcmp(argv[1], "-n") == 0) {
        newline = 0;
        start = 2;
    }
    for (int i = start; i < argc; i++) {
        printf("%s", argv[i]);
        if (i < argc - 1) {
            putchar(' ');
        }
    }
    if (newline) {
        putchar('\n');
    }
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
            sh_var_set(argv[i], eq + 1, 1);
            *eq = '=';
        } else {
            const char *existing = sh_var_get(argv[i]);
            sh_var_set(argv[i], existing ? existing : "", 1);
        }
    }
    return 0;
}

static int builtin_unset(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        sh_var_unset(argv[i]);
    }
    return 0;
}

static int builtin_set(int argc, char **argv) {
    (void)argc;
    (void)argv;
    for (int i = 0; i < sh_var_count; i++) {
        printf("%s=%s\n", sh_vars[i].name, sh_vars[i].value);
    }
    return 0;
}

static int builtin_shift(int argc, char **argv) {
    int n = 1;
    if (argc > 1) {
        n = atoi(argv[1]);
        if (n < 1) {
            n = 1;
        }
    }
    if (sh_argc <= 1) {
        return 0;
    }
    if (n >= sh_argc) {
        sh_argc = 1;
        sh_argv[1] = NULL;
        return 0;
    }
    for (int i = 1; i + n < sh_argc; i++) {
        strncpy(sh_argv_storage[i], sh_argv_storage[i + n], SH_MAX_LINE - 1);
        sh_argv_storage[i][SH_MAX_LINE - 1] = '\0';
    }
    sh_argc -= n;
    sh_argv[sh_argc] = NULL;
    return 0;
}

static int builtin_source(int argc, char **argv) {
    if (argc < 2) {
        printf("sh: %s: filename argument required\n", argv[0]);
        return 2;
    }
    FILE *fp = fopen(argv[1], "r");
    if (!fp) {
        printf("sh: %s: No such file or directory\n", argv[1]);
        return 127;
    }
    int status = run_script_stream(fp);
    fclose(fp);
    return status;
}

static int builtin_help(void) {
    puts("Keira Userspace Shell Console (Ring 3 v" KEIRA_VERSION ")");
    putchar('\n');
    puts("Shell Built-ins:");
    puts("  cd [dir]       Change current working directory");
    puts("  pwd            Print working directory");
    puts("  echo [-n]      Echo arguments with variable expansion");
    puts("  export [var]   Set or export environment variable");
    puts("  unset <var>    Unset shell or environment variable");
    puts("  set            List active shell variables");
    puts("  shift [n]      Shift positional parameters");
    puts("  test / [       Evaluate conditional expression");
    puts("  source / .     Execute script in current shell context");
    puts("  history        Display command execution history");
    puts("  jobs           List background and suspended jobs");
    puts("  fg [%job]      Bring background job to foreground");
    puts("  bg [%job]      Resume stopped job in background");
    puts("  help           Display shell command reference");
    puts("  exit [code]    Exit userspace shell\n");
    puts("Scripting Language Constructs:");
    puts("  if CMD; then CMDS; [elif CMD; then CMDS;] [else CMDS;] fi");
    puts("  for VAR in WORDS; do CMDS; done");
    puts("  while CMD; do CMDS; done");
    puts("  Variables: $VAR, ${VAR}, $?, $$, $#, $0..$9, $*");
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

static int is_sh_binary(const char *cmd) {
    if (!cmd) {
        return 0;
    }
    const char *base = strrchr(cmd, '/');
    const char *name = base ? base + 1 : cmd;
    return (strcmp(name, "sh") == 0 || strcmp(name, "sh.elf") == 0);
}

static int execute_single_command(int argc, char **argv) {
    if (argc == 0 || !argv[0]) {
        return 0;
    }

    int is_background = 0;
    if (argc > 0 && strcmp(argv[argc - 1], "&") == 0) {
        is_background = 1;
        argv[argc - 1] = NULL;
        argc--;
    } else if (argc > 0) {
        size_t last_len = strlen(argv[argc - 1]);
        if (last_len > 1 && argv[argc - 1][last_len - 1] == '&') {
            argv[argc - 1][last_len - 1] = '\0';
            is_background = 1;
        }
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

    if (clean_argc > 0 && strcmp(clean_argv[clean_argc - 1], "&") == 0) {
        is_background = 1;
        clean_argv[clean_argc - 1] = NULL;
        clean_argc--;
    }

    if (clean_argc == 0) {
        return 0;
    }

    /* Check if command is a standalone assignment: VAR=value */
    if (clean_argc == 1 && strchr(clean_argv[0], '=')) {
        char *eq = strchr(clean_argv[0], '=');
        *eq = '\0';
        int is_valid_name = 1;
        if (!((clean_argv[0][0] >= 'a' && clean_argv[0][0] <= 'z') ||
              (clean_argv[0][0] >= 'A' && clean_argv[0][0] <= 'Z') || clean_argv[0][0] == '_')) {
            is_valid_name = 0;
        }
        for (char *c = clean_argv[0] + 1; *c; c++) {
            if (!((*c >= 'a' && *c <= 'z') || (*c >= 'A' && *c <= 'Z') ||
                  (*c >= '0' && *c <= '9') || *c == '_')) {
                is_valid_name = 0;
                break;
            }
        }
        if (is_valid_name) {
            sh_var_set(clean_argv[0], eq + 1, 0);
            *eq = '=';
            return 0;
        }
        *eq = '=';
    }

    char full_cmd[SH_MAX_LINE];
    full_cmd[0] = '\0';
    for (int i = 0; i < clean_argc; i++) {
        if (i > 0) {
            strncat(full_cmd, " ", sizeof(full_cmd) - strlen(full_cmd) - 1);
        }
        strncat(full_cmd, clean_argv[i], sizeof(full_cmd) - strlen(full_cmd) - 1);
    }
    if (is_background) {
        strncat(full_cmd, " &", sizeof(full_cmd) - strlen(full_cmd) - 1);
    }

    int saved_stdout = -1;
    int saved_stdin = -1;

    if (out_file) {
        int flags = O_WRONLY | O_CREAT | (out_append ? O_APPEND : O_TRUNC);
        int fd_out = open(out_file, flags, 0644);
        if (fd_out < 0) {
            printf("sh: cannot create %s\n", out_file);
            return 1;
        }
        saved_stdout = dup(STDOUT_FILENO);
        dup2(fd_out, STDOUT_FILENO);
        close(fd_out);
    }

    if (in_file) {
        int fd_in = open(in_file, O_RDONLY, 0);
        if (fd_in < 0) {
            printf("sh: %s: No such file or directory\n", in_file);
            if (saved_stdout >= 0) {
                dup2(saved_stdout, STDOUT_FILENO);
                close(saved_stdout);
            }
            return 1;
        }
        saved_stdin = dup(STDIN_FILENO);
        dup2(fd_in, STDIN_FILENO);
        close(fd_in);
    }

    int ret_status = 0;

    if (strcmp(clean_argv[0], "cd") == 0) {
        ret_status = builtin_cd(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "pwd") == 0) {
        ret_status = builtin_pwd();
    } else if (strcmp(clean_argv[0], "echo") == 0) {
        ret_status = builtin_echo(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "export") == 0) {
        ret_status = builtin_export(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "unset") == 0) {
        ret_status = builtin_unset(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "set") == 0) {
        ret_status = builtin_set(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "shift") == 0) {
        ret_status = builtin_shift(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "test") == 0 || strcmp(clean_argv[0], "[") == 0) {
        ret_status = builtin_test(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], ".") == 0 || strcmp(clean_argv[0], "source") == 0) {
        ret_status = builtin_source(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "break") == 0) {
        sh_loop_break = 1;
        ret_status = 0;
    } else if (strcmp(clean_argv[0], "continue") == 0) {
        sh_loop_continue = 1;
        ret_status = 0;
    } else if (strcmp(clean_argv[0], "return") == 0) {
        sh_return_flag = 1;
        ret_status = (clean_argc > 1) ? atoi(clean_argv[1]) : 0;
    } else if (strcmp(clean_argv[0], "true") == 0 || strcmp(clean_argv[0], ":") == 0) {
        ret_status = 0;
    } else if (strcmp(clean_argv[0], "false") == 0) {
        ret_status = 1;
    } else if (strcmp(clean_argv[0], "clear") == 0) {
        printf("\x1b[2J\x1b[H");
        ret_status = 0;
    } else if (strcmp(clean_argv[0], "history") == 0) {
        ret_status = builtin_history();
    } else if (strcmp(clean_argv[0], "jobs") == 0) {
        ret_status = builtin_jobs(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "fg") == 0) {
        ret_status = builtin_fg(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "bg") == 0) {
        ret_status = builtin_bg(clean_argc, clean_argv);
    } else if (strcmp(clean_argv[0], "help") == 0) {
        ret_status = builtin_help();
    } else if (strcmp(clean_argv[0], "exit") == 0) {
        int code = (clean_argc > 1) ? atoi(clean_argv[1]) : 0;
        exit(code);
    } else if (is_sh_binary(clean_argv[0]) && clean_argc == 1 && isatty(STDIN_FILENO)) {
        ret_status = 0;
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
                setpgid(pid, pid);
                if (is_background) {
                    job_t *job = job_add(pid, pid, full_cmd);
                    if (job) {
                        printf("[%d] %d\n", job->id, (int)pid);
                        fflush(stdout);
                    }
                    ret_status = 0;
                } else {
                    int status = 0;
                    waitpid(pid, &status, 0);
                    if (WIFEXITED(status)) {
                        ret_status = WEXITSTATUS(status);
                    } else {
                        ret_status = 0;
                    }
                    if (is_sh_binary(resolved_path) && clean_argc == 1 && isatty(STDIN_FILENO)) {
                        exit(ret_status);
                    }
                }
            }
        }
    }

    fflush(stdout);
    fflush(stderr);

    if (saved_stdout >= 0) {
        dup2(saved_stdout, STDOUT_FILENO);
        close(saved_stdout);
    }
    if (saved_stdin >= 0) {
        dup2(saved_stdin, STDIN_FILENO);
        close(saved_stdin);
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
        char expanded[SH_MAX_LINE * 2];
        expand_parameters(pipes[0], expanded, sizeof(expanded));
        char *argv[SH_MAX_ARGS];
        int argc = tokenize_line(expanded, argv, SH_MAX_ARGS);
        return execute_single_command(argc, argv);
    }

    int saved_stdin = dup(STDIN_FILENO);
    int saved_stdout = dup(STDOUT_FILENO);
    if (saved_stdin < 0 || saved_stdout < 0) {
        if (saved_stdin >= 0) {
            close(saved_stdin);
        }
        if (saved_stdout >= 0) {
            close(saved_stdout);
        }
        return 1;
    }

    int prev_in = -1;
    int pipe_status = 0;

    for (int i = 0; i < num_pipes; i++) {
        int pipefd[2] = {-1, -1};
        if (i < num_pipes - 1) {
            if (pipe(pipefd) < 0) {
                pipe_status = 1;
                break;
            }
        }

        if (prev_in != -1) {
            dup2(prev_in, STDIN_FILENO);
            close(prev_in);
            prev_in = -1;
        }

        if (i < num_pipes - 1) {
            dup2(pipefd[1], STDOUT_FILENO);
            close(pipefd[1]);
        } else {
            dup2(saved_stdout, STDOUT_FILENO);
        }

        char expanded[SH_MAX_LINE * 2];
        expand_parameters(pipes[i], expanded, sizeof(expanded));
        char *argv[SH_MAX_ARGS];
        int argc = tokenize_line(expanded, argv, SH_MAX_ARGS);
        if (argc > 0) {
            pipe_status = execute_single_command(argc, argv);
        }

        if (i < num_pipes - 1) {
            prev_in = pipefd[0];
            dup2(saved_stdout, STDOUT_FILENO);
        }
    }

    if (prev_in != -1) {
        close(prev_in);
    }

    if (saved_stdin >= 0) {
        dup2(saved_stdin, STDIN_FILENO);
        close(saved_stdin);
    }
    if (saved_stdout >= 0) {
        dup2(saved_stdout, STDOUT_FILENO);
        close(saved_stdout);
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

static int is_word_boundary(char c) {
    return c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == ';' || c == '\0';
}

static int is_kw(const char *start, const char *p, const char *kw) {
    size_t kw_len = strlen(kw);
    if (p > start && !is_word_boundary(*(p - 1))) {
        return 0;
    }
    if (strncmp(p, kw, kw_len) == 0 && is_word_boundary(p[kw_len])) {
        return 1;
    }
    return 0;
}

static char *find_matching_kw(char *str, const char *kw1, const char *kw2, const char *kw3) {
    char *p = str;
    char in_quote = '\0';
    int depth = 0;

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (in_quote == '\0') {
            if (*p == '#' && (p == str || is_word_boundary(*(p - 1)))) {
                while (*p && *p != '\n') {
                    p++;
                }
                continue;
            }
            if (depth == 0) {
                if (kw1 && is_kw(str, p, kw1)) {
                    return p;
                }
                if (kw2 && is_kw(str, p, kw2)) {
                    return p;
                }
                if (kw3 && is_kw(str, p, kw3)) {
                    return p;
                }
            }
            if (is_kw(str, p, "if") || is_kw(str, p, "for") || is_kw(str, p, "while")) {
                depth++;
            } else if (is_kw(str, p, "fi") || is_kw(str, p, "done")) {
                if (depth > 0) {
                    depth--;
                }
            }
        }
        p++;
    }
    return NULL;
}

static void trim_trailing_delimiters(char *start, char *end) {
    while (end > start && (is_space(*(end - 1)) || *(end - 1) == ';')) {
        end--;
    }
    *end = '\0';
}

static int execute_if_statement(char *cmd) {
    char *p = cmd + 2;
    while (*p && is_space(*p)) {
        p++;
    }

    char *then_kw = find_matching_kw(p, "then", NULL, NULL);
    if (!then_kw) {
        printf("sh: syntax error: `then' expected\n");
        return 2;
    }

    trim_trailing_delimiters(p, then_kw);
    char *cond_str = p;
    char *after_then = then_kw + 4;
    while (*after_then && (is_space(*after_then) || *after_then == ';')) {
        after_then++;
    }

    char *end_branch = find_matching_kw(after_then, "elif", "else", "fi");
    if (!end_branch) {
        printf("sh: syntax error: unexpected EOF waiting for `fi'\n");
        return 2;
    }

    int cond_status = run_command_string(cond_str);

    if (cond_status == 0) {
        char saved = *end_branch;
        *end_branch = '\0';
        int then_status = run_command_string(after_then);
        *end_branch = saved;
        return then_status;
    }

    if (is_kw(after_then, end_branch, "elif")) {
        char *elif_str = end_branch + 2;
        return execute_if_statement(elif_str);
    } else if (is_kw(after_then, end_branch, "else")) {
        char *after_else = end_branch + 4;
        while (*after_else && (is_space(*after_else) || *after_else == ';')) {
            after_else++;
        }
        char *fi_kw = find_matching_kw(after_else, "fi", NULL, NULL);
        if (!fi_kw) {
            printf("sh: syntax error: unexpected EOF waiting for `fi'\n");
            return 2;
        }
        *fi_kw = '\0';
        return run_command_string(after_else);
    }

    return 0;
}

static int execute_for_statement(char *cmd) {
    char *p = cmd + 3;
    while (*p && is_space(*p)) {
        p++;
    }

    char var_name[64];
    size_t v_idx = 0;
    while (*p && !is_space(*p) && *p != ';' && v_idx + 1 < sizeof(var_name)) {
        var_name[v_idx++] = *p++;
    }
    var_name[v_idx] = '\0';

    while (*p && is_space(*p)) {
        p++;
    }
    if (strncmp(p, "in", 2) != 0 || !is_word_boundary(p[2])) {
        printf("sh: syntax error: `in' expected in for loop\n");
        return 2;
    }
    p += 2;
    while (*p && is_space(*p)) {
        p++;
    }

    char *do_kw = find_matching_kw(p, "do", NULL, NULL);
    if (!do_kw) {
        printf("sh: syntax error: `do' expected in for loop\n");
        return 2;
    }

    trim_trailing_delimiters(p, do_kw);
    char *words_raw = p;
    char *after_do = do_kw + 2;
    while (*after_do && (is_space(*after_do) || *after_do == ';')) {
        after_do++;
    }

    char *done_kw = find_matching_kw(after_do, "done", NULL, NULL);
    if (!done_kw) {
        printf("sh: syntax error: unexpected EOF waiting for `done'\n");
        return 2;
    }
    *done_kw = '\0';
    char *body_str = after_do;

    char expanded_words[SH_MAX_LINE * 2];
    expand_parameters(words_raw, expanded_words, sizeof(expanded_words));

    char *words[SH_MAX_ARGS];
    int word_count = tokenize_line(expanded_words, words, SH_MAX_ARGS);

    int status = 0;
    for (int i = 0; i < word_count; i++) {
        sh_var_set(var_name, words[i], 0);

        char body_copy[SH_MAX_LINE * 2];
        strncpy(body_copy, body_str, sizeof(body_copy) - 1);
        body_copy[sizeof(body_copy) - 1] = '\0';

        status = run_command_string(body_copy);

        if (sh_loop_break) {
            sh_loop_break = 0;
            break;
        }
        if (sh_loop_continue) {
            sh_loop_continue = 0;
            continue;
        }
        if (sh_return_flag) {
            break;
        }
    }

    return status;
}

static int execute_while_statement(char *cmd) {
    char *p = cmd + 5;
    while (*p && is_space(*p)) {
        p++;
    }

    char *do_kw = find_matching_kw(p, "do", NULL, NULL);
    if (!do_kw) {
        printf("sh: syntax error: `do' expected in while loop\n");
        return 2;
    }

    trim_trailing_delimiters(p, do_kw);
    char *cond_str = p;
    char *after_do = do_kw + 2;
    while (*after_do && (is_space(*after_do) || *after_do == ';')) {
        after_do++;
    }

    char *done_kw = find_matching_kw(after_do, "done", NULL, NULL);
    if (!done_kw) {
        printf("sh: syntax error: unexpected EOF waiting for `done'\n");
        return 2;
    }
    *done_kw = '\0';
    char *body_str = after_do;

    int status = 0;
    while (1) {
        char cond_copy[SH_MAX_LINE * 2];
        strncpy(cond_copy, cond_str, sizeof(cond_copy) - 1);
        cond_copy[sizeof(cond_copy) - 1] = '\0';

        int cond_status = run_command_string(cond_copy);
        if (cond_status != 0) {
            break;
        }

        char body_copy[SH_MAX_LINE * 2];
        strncpy(body_copy, body_str, sizeof(body_copy) - 1);
        body_copy[sizeof(body_copy) - 1] = '\0';

        status = run_command_string(body_copy);

        if (sh_loop_break) {
            sh_loop_break = 0;
            break;
        }
        if (sh_loop_continue) {
            sh_loop_continue = 0;
            continue;
        }
        if (sh_return_flag) {
            break;
        }
    }

    return status;
}

static int execute_top_statement(char *stmt) {
    while (*stmt && is_space(*stmt)) {
        stmt++;
    }
    if (!*stmt) {
        return 0;
    }

    if (is_kw(stmt, stmt, "if")) {
        return execute_if_statement(stmt);
    }
    if (is_kw(stmt, stmt, "for")) {
        return execute_for_statement(stmt);
    }
    if (is_kw(stmt, stmt, "while")) {
        return execute_while_statement(stmt);
    }

    return execute_chained_statement(stmt);
}

static int calculate_block_depth_delta(const char *line) {
    int delta = 0;
    const char *p = line;
    char in_quote = '\0';

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (in_quote == '\0') {
            if (*p == '#' && (p == line || is_word_boundary(*(p - 1)))) {
                break;
            }
            if (is_kw(line, p, "if") || is_kw(line, p, "for") || is_kw(line, p, "while")) {
                delta++;
            } else if (is_kw(line, p, "fi") || is_kw(line, p, "done")) {
                delta--;
            }
        }
        p++;
    }
    return delta;
}

static int run_command_string(char *cmd_str) {
    char *s = cmd_str;
    char q = '\0';
    while (*s) {
        if ((*s == '"' || *s == '\'') && q == '\0') {
            q = *s;
        } else if (*s == q) {
            q = '\0';
        } else if (*s == '#' && q == '\0' && (s == cmd_str || is_word_boundary(*(s - 1)))) {
            while (*s && *s != '\n') {
                *s++ = ' ';
            }
            continue;
        }
        s++;
    }

    char *p = cmd_str;
    char *stmt_start = p;
    char in_quote = '\0';
    int block_depth = 0;
    int status = 0;

    while (*p) {
        if ((*p == '"' || *p == '\'') && in_quote == '\0') {
            in_quote = *p;
        } else if (*p == in_quote) {
            in_quote = '\0';
        } else if (in_quote == '\0') {
            if (is_kw(cmd_str, p, "if") || is_kw(cmd_str, p, "for") || is_kw(cmd_str, p, "while")) {
                block_depth++;
            } else if (is_kw(cmd_str, p, "fi") || is_kw(cmd_str, p, "done")) {
                if (block_depth > 0) {
                    block_depth--;
                }
            } else if (block_depth == 0 && (*p == ';' || *p == '\n')) {
                *p = '\0';
                while (*stmt_start && is_space(*stmt_start)) {
                    stmt_start++;
                }
                if (*stmt_start) {
                    status = execute_top_statement(stmt_start);
                    if (sh_return_flag || sh_loop_break || sh_loop_continue) {
                        return status;
                    }
                }
                stmt_start = p + 1;
            }
        }
        p++;
    }

    while (*stmt_start && is_space(*stmt_start)) {
        stmt_start++;
    }
    if (*stmt_start) {
        status = execute_top_statement(stmt_start);
    }

    return status;
}

static int run_script_stream(FILE *fp) {
    char block_buf[SH_MAX_LINE * 4];
    block_buf[0] = '\0';
    size_t block_len = 0;
    int block_depth = 0;

    char line_buf[SH_MAX_LINE];
    int status = 0;

    while (fgets(line_buf, sizeof(line_buf), fp)) {
        job_reap();
        size_t len = strlen(line_buf);
        while (len > 0 && (line_buf[len - 1] == '\n' || line_buf[len - 1] == '\r')) {
            line_buf[--len] = '\0';
        }
        if (len == 0 && block_depth == 0) {
            continue;
        }

        int delta = calculate_block_depth_delta(line_buf);
        block_depth += delta;
        if (block_depth < 0) {
            block_depth = 0;
        }

        if (block_len + len + 2 < sizeof(block_buf)) {
            if (block_len > 0) {
                block_buf[block_len++] = '\n';
                block_buf[block_len] = '\0';
            }
            memcpy(&block_buf[block_len], line_buf, len);
            block_len += len;
            block_buf[block_len] = '\0';
        }

        if (block_depth == 0 && block_len > 0) {
            status = run_command_string(block_buf);
            block_buf[0] = '\0';
            block_len = 0;
            if (sh_return_flag) {
                sh_return_flag = 0;
                break;
            }
        }
    }

    if (block_len > 0) {
        status = run_command_string(block_buf);
    }

    return status;
}

static int run_script_file(const char *filename) {
    FILE *fp = fopen(filename, "r");
    if (!fp) {
        printf("sh: %s: No such file or directory\n", filename);
        return 127;
    }
    int status = run_script_stream(fp);
    fclose(fp);
    return status;
}

static int sh_readline(char *buf, size_t max_len) {
    if (!isatty(STDIN_FILENO)) {
        if (!fgets(buf, (int)max_len, stdin)) {
            return -1;
        }
        size_t len = strlen(buf);
        while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r')) {
            buf[--len] = '\0';
        }
        return (int)len;
    }

    struct termios orig_term;
    int has_term = (tcgetattr(STDIN_FILENO, &orig_term) == 0);
    if (has_term) {
        struct termios raw_term = orig_term;
        raw_term.c_lflag &= ~(ICANON | ECHO);
        tcsetattr(STDIN_FILENO, TCSANOW, &raw_term);
    }

    size_t line_len = 0;
    buf[0] = '\0';
    char draft_buf[SH_MAX_LINE];
    draft_buf[0] = '\0';
    int hist_index = -1;

    while (1) {
        char ch = 0;
        ssize_t n = read(STDIN_FILENO, &ch, 1);
        if (n <= 0) {
            if (has_term) {
                tcsetattr(STDIN_FILENO, TCSANOW, &orig_term);
            }
            return -1;
        }

        if (ch == 0x04) {
            if (line_len == 0) {
                if (has_term) {
                    tcsetattr(STDIN_FILENO, TCSANOW, &orig_term);
                }
                return -1;
            }
            continue;
        }

        if (ch == 0x03) {
            buf[0] = '\0';
            putchar('\n');
            fflush(stdout);
            if (has_term) {
                tcsetattr(STDIN_FILENO, TCSANOW, &orig_term);
            }
            return 0;
        }

        if (ch == '\r' || ch == '\n') {
            buf[line_len] = '\0';
            putchar('\n');
            fflush(stdout);
            if (line_len > 0) {
                sh_history_push(buf);
            }
            if (has_term) {
                tcsetattr(STDIN_FILENO, TCSANOW, &orig_term);
            }
            return (int)line_len;
        }

        if (ch == 0x08 || ch == 0x7F) {
            if (line_len > 0) {
                line_len--;
                buf[line_len] = '\0';
                printf("\b \b");
                fflush(stdout);
            }
            continue;
        }

        if ((unsigned char)ch == 0x1B) {
            char seq[2];
            if (read(STDIN_FILENO, &seq[0], 1) == 1 && seq[0] == '[') {
                if (read(STDIN_FILENO, &seq[1], 1) == 1) {
                    if (seq[1] == 'A') {
                        if (sh_history_count > 0) {
                            if (hist_index == -1) {
                                strncpy(draft_buf, buf, sizeof(draft_buf) - 1);
                                draft_buf[sizeof(draft_buf) - 1] = '\0';
                                hist_index = sh_history_count - 1;
                            } else if (hist_index > 0) {
                                int oldest = (sh_history_count > SH_HISTORY_MAX)
                                                 ? (sh_history_count - SH_HISTORY_MAX)
                                                 : 0;
                                if (hist_index > oldest) {
                                    hist_index--;
                                }
                            }
                            int idx = hist_index % SH_HISTORY_MAX;
                            strncpy(buf, sh_history[idx], max_len - 1);
                            buf[max_len - 1] = '\0';
                            line_len = strlen(buf);

                            printf("\r\x1b[2K");
                            print_shell_prompt();
                            printf("%s", buf);
                            fflush(stdout);
                        }
                    } else if (seq[1] == 'B') {
                        if (hist_index != -1) {
                            if (hist_index < sh_history_count - 1) {
                                hist_index++;
                                int idx = hist_index % SH_HISTORY_MAX;
                                strncpy(buf, sh_history[idx], max_len - 1);
                                buf[max_len - 1] = '\0';
                                line_len = strlen(buf);
                            } else {
                                hist_index = -1;
                                strncpy(buf, draft_buf, max_len - 1);
                                buf[max_len - 1] = '\0';
                                line_len = strlen(buf);
                            }

                            printf("\r\x1b[2K");
                            print_shell_prompt();
                            printf("%s", buf);
                            fflush(stdout);
                        }
                    }
                }
            }
            continue;
        }

        if ((unsigned char)ch >= 32 && (unsigned char)ch <= 126) {
            if (line_len + 1 < max_len) {
                buf[line_len++] = ch;
                buf[line_len] = '\0';
                putchar(ch);
                fflush(stdout);
            }
        }
    }
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
        sh_argc = 1;
        strncpy(sh_argv_storage[0], "sh", SH_MAX_LINE - 1);
        sh_argv_storage[0][SH_MAX_LINE - 1] = '\0';
        sh_argv[0] = sh_argv_storage[0];
        for (int i = 3; i < argc && sh_argc < SH_MAX_ARGS - 1; i++) {
            strncpy(sh_argv_storage[sh_argc], argv[i], SH_MAX_LINE - 1);
            sh_argv_storage[sh_argc][SH_MAX_LINE - 1] = '\0';
            sh_argv[sh_argc] = sh_argv_storage[sh_argc];
            sh_argc++;
        }
        sh_argv[sh_argc] = NULL;

        return run_command_string(argv[2]);
    }

    if (argc >= 2 && argv[1][0] != '-') {
        sh_argc = 0;
        for (int i = 1; i < argc && sh_argc < SH_MAX_ARGS - 1; i++) {
            strncpy(sh_argv_storage[sh_argc], argv[i], SH_MAX_LINE - 1);
            sh_argv_storage[sh_argc][SH_MAX_LINE - 1] = '\0';
            sh_argv[sh_argc] = sh_argv_storage[sh_argc];
            sh_argc++;
        }
        sh_argv[sh_argc] = NULL;
        return run_script_file(argv[1]);
    }

    sh_argc = 1;
    strncpy(sh_argv_storage[0], "sh", SH_MAX_LINE - 1);
    sh_argv_storage[0][SH_MAX_LINE - 1] = '\0';
    sh_argv[0] = sh_argv_storage[0];
    sh_argv[1] = NULL;

    char block_buf[SH_MAX_LINE * 4];
    block_buf[0] = '\0';
    size_t block_len = 0;
    int block_depth = 0;

    char line_buf[SH_MAX_LINE];
    while (1) {
        job_reap();
        if (block_depth == 0) {
            print_shell_prompt();
        } else {
            printf("> ");
            fflush(stdout);
        }

        int n = sh_readline(line_buf, sizeof(line_buf));
        if (n < 0) {
            puts("exit");
            break;
        }
        if (n == 0 && block_depth == 0) {
            continue;
        }

        int delta = calculate_block_depth_delta(line_buf);
        block_depth += delta;
        if (block_depth < 0) {
            block_depth = 0;
        }

        size_t len = strlen(line_buf);
        if (block_len + len + 2 < sizeof(block_buf)) {
            if (block_len > 0) {
                block_buf[block_len++] = '\n';
                block_buf[block_len] = '\0';
            }
            memcpy(&block_buf[block_len], line_buf, len);
            block_len += len;
            block_buf[block_len] = '\0';
        }

        if (block_depth == 0 && block_len > 0) {
            run_command_string(block_buf);
            block_buf[0] = '\0';
            block_len = 0;
        }
    }

    return last_exit_status;
}
