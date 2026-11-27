/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Expression Evaluator Utility (test / [)
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
#include <sys/stat.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: test EXPRESSION");
    puts("       [ EXPRESSION ]");
    puts("");
    puts("Description:");
    puts("  Evaluate conditional expression and exit with status 0 (true) or 1 (false).");
    puts("");
    puts("File Operators:");
    puts("  -e FILE        True if file exists");
    puts("  -f FILE        True if file exists and is a regular file");
    puts("  -d FILE        True if file exists and is a directory");
    puts("  -s FILE        True if file exists and has size greater than zero");
    puts("  -r FILE        True if file exists and read permission is granted");
    puts("  -w FILE        True if file exists and write permission is granted");
    puts("  -x FILE        True if file exists and execute permission is granted");
    puts("");
    puts("String Operators:");
    puts("  -z STRING      True if length of STRING is zero");
    puts("  -n STRING      True if length of STRING is non-zero");
    puts("  STRING1 = STRING2   True if strings are identical");
    puts("  STRING1 != STRING2  True if strings are not identical");
    puts("");
    puts("Integer Comparisons:");
    puts("  INT1 -eq INT2  True if integers are equal");
    puts("  INT1 -ne INT2  True if integers are not equal");
    puts("  INT1 -lt INT2  True if INT1 is less than INT2");
    puts("  INT1 -le INT2  True if INT1 is less than or equal to INT2");
    puts("  INT1 -gt INT2  True if INT1 is greater than INT2");
    puts("  INT1 -ge INT2  True if INT1 is greater than or equal to INT2");
    puts("");
    puts("Logical Operators:");
    puts("  ! EXPR         Invert expression truth value");
    puts("  EXPR1 -a EXPR2 True if both EXPR1 and EXPR2 are true");
    puts("  EXPR1 -o EXPR2 True if either EXPR1 or EXPR2 is true");
    puts("");
    puts("Options:");
    puts("  -h, --help     Display this help reference and exit");
}

static int eval_test(int argc, char **argv) {
    if (argc <= 0) {
        return 1;
    }

    /* Check for logical OR (-o) with lowest precedence */
    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], "-o") == 0 && i > 0 && i < argc - 1) {
            int left = eval_test(i, argv);
            if (left == 0) {
                return 0;
            }
            return eval_test(argc - i - 1, argv + i + 1);
        }
    }

    /* Check for logical AND (-a) */
    for (int i = 0; i < argc; i++) {
        if (strcmp(argv[i], "-a") == 0 && i > 0 && i < argc - 1) {
            int left = eval_test(i, argv);
            if (left != 0) {
                return 1;
            }
            return eval_test(argc - i - 1, argv + i + 1);
        }
    }

    /* Negation operator (!) */
    if (strcmp(argv[0], "!") == 0) {
        return eval_test(argc - 1, argv + 1) == 0 ? 1 : 0;
    }

    /* Parentheses grouping */
    if (argc >= 2 && strcmp(argv[0], "(") == 0 && strcmp(argv[argc - 1], ")") == 0) {
        return eval_test(argc - 2, argv + 1);
    }

    /* 1 argument: true if non-empty string */
    if (argc == 1) {
        return argv[0][0] != '\0' ? 0 : 1;
    }

    /* 2 arguments: unary operators */
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

    /* 3 arguments: binary comparisons */
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

int main(int argc, char **argv) {
    const char *prog = argv[0];
    const char *slash = strrchr(prog, '/');
    if (slash) {
        prog = slash + 1;
    }

    int is_bracket = (strcmp(prog, "[") == 0 || strcmp(prog, "[.elf") == 0);

    if (is_bracket) {
        if (argc < 2 || strcmp(argv[argc - 1], "]") != 0) {
            fprintf(stderr, "[: missing `]'\n");
            return 2;
        }
        argc--;
    } else {
        if (argc == 2 && (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0)) {
            print_usage();
            return 0;
        }
    }

    return eval_test(argc - 1, argv + 1);
}
