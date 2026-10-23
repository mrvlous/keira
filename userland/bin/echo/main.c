/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Argument Echo Utility (echo)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: echo [SHORT-OPTION]... [STRING]...");
    puts("");
    puts("Description:");
    puts("  Echo the STRING(s) to standard output.");
    puts("");
    puts("Options:");
    puts("  -n             Do not output the trailing newline");
    puts("  -e             Enable interpretation of backslash escapes");
    puts("  -E             Disable interpretation of backslash escapes (default)");
    puts("  -h, --help     Display this help reference and exit");
}

static void print_escaped(const char *s) {
    while (*s) {
        if (*s == '\\' && *(s + 1)) {
            s++;
            switch (*s) {
            case 'n':
                putchar('\n');
                break;
            case 't':
                putchar('\t');
                break;
            case 'r':
                putchar('\r');
                break;
            case '\\':
                putchar('\\');
                break;
            case '0':
                putchar('\0');
                break;
            case 'a':
                putchar('\a');
                break;
            case 'b':
                putchar('\b');
                break;
            default:
                putchar('\\');
                putchar(*s);
                break;
            }
        } else {
            putchar(*s);
        }
        s++;
    }
}

int main(int argc, char **argv) {
    int opt_newline = 1;
    int opt_escape = 0;
    int arg_start = 1;

    while (arg_start < argc && argv[arg_start][0] == '-' && argv[arg_start][1] != '\0') {
        if (strcmp(argv[arg_start], "--help") == 0) {
            print_usage();
            return 0;
        }

        const char *p = argv[arg_start] + 1;
        int valid_flag = 1;
        int n_flag = 0;
        int e_flag = 0;
        int big_e_flag = 0;

        while (*p) {
            if (*p == 'n') {
                n_flag = 1;
            } else if (*p == 'e') {
                e_flag = 1;
                big_e_flag = 0;
            } else if (*p == 'E') {
                big_e_flag = 1;
                e_flag = 0;
            } else {
                valid_flag = 0;
                break;
            }
            p++;
        }

        if (!valid_flag) {
            break;
        }

        if (n_flag) {
            opt_newline = 0;
        }
        if (e_flag) {
            opt_escape = 1;
        }
        if (big_e_flag) {
            opt_escape = 0;
        }
        arg_start++;
    }

    for (int i = arg_start; i < argc; i++) {
        if (opt_escape) {
            print_escaped(argv[i]);
        } else {
            printf("%s", argv[i]);
        }
        if (i + 1 < argc) {
            putchar(' ');
        }
    }

    if (opt_newline) {
        putchar('\n');
    }

    return 0;
}
