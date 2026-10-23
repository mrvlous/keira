/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 File Mode Modification Utility (chmod)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: chmod [OPTION]... MODE[,MODE]... FILE...");
    puts("   or: chmod [OPTION]... OCTAL-MODE FILE...");
    puts("");
    puts("Description:");
    puts("  Change the mode of each FILE to MODE.");
    puts("");
    puts("Options:");
    puts("  -R, --recursive  change files and directories recursively");
    puts("  -v, --verbose    output a diagnostic for every file processed");
    puts("  -h, --help       display this help message and exit");
    puts("");
    puts("Modes:");
    puts("  Octal:    755, 644, 600, etc.");
    puts("  Symbolic: [ugoa]*[-+=][rwx]+ (e.g. +x, -w, u+rwx, go-w)");
}

static int parse_octal_mode(const char *str, mode_t *mode_out) {
    char *endptr;
    long val = strtol(str, &endptr, 8);
    if (*endptr != '\0' || val < 0 || val > 07777) {
        return -1;
    }
    *mode_out = (mode_t)val;
    return 0;
}

static int apply_symbolic_mode(const char *spec, mode_t current_mode, mode_t *new_mode_out) {
    mode_t mode = current_mode & 07777;
    const char *p = spec;

    while (*p) {
        mode_t who_mask = 0;
        while (*p == 'u' || *p == 'g' || *p == 'o' || *p == 'a') {
            if (*p == 'u')
                who_mask |= 0700;
            if (*p == 'g')
                who_mask |= 0070;
            if (*p == 'o')
                who_mask |= 0007;
            if (*p == 'a')
                who_mask |= 0777;
            p++;
        }
        if (who_mask == 0) {
            who_mask = 0777;
        }

        if (*p != '+' && *p != '-' && *p != '=') {
            return -1;
        }
        char op = *p++;

        mode_t perm_bits = 0;
        while (*p == 'r' || *p == 'w' || *p == 'x') {
            if (*p == 'r')
                perm_bits |= 0444;
            if (*p == 'w')
                perm_bits |= 0222;
            if (*p == 'x')
                perm_bits |= 0111;
            p++;
        }

        mode_t affected = perm_bits & who_mask;
        if (op == '+') {
            mode |= affected;
        } else if (op == '-') {
            mode &= ~affected;
        } else if (op == '=') {
            mode = (mode & ~who_mask) | affected;
        }

        if (*p == ',') {
            p++;
        } else if (*p != '\0') {
            return -1;
        }
    }

    *new_mode_out = mode;
    return 0;
}

static int chmod_path(const char *path, const char *mode_str, int is_octal, mode_t octal_val,
                      int recursive, int verbose) {
    struct stat st;
    if (stat(path, &st) != 0) {
        printf("chmod: cannot access '%s': ", path);
        if (errno == ENOENT) {
            puts("No such file or directory");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else {
            puts("Error accessing file");
        }
        return 1;
    }

    mode_t target_mode = 0;
    if (is_octal) {
        target_mode = octal_val;
    } else {
        if (apply_symbolic_mode(mode_str, st.st_mode, &target_mode) != 0) {
            printf("chmod: invalid mode: '%s'\n", mode_str);
            return 1;
        }
    }

    if (chmod(path, target_mode) != 0) {
        printf("chmod: changing permissions of '%s': ", path);
        if (errno == EPERM) {
            puts("Operation not permitted");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else {
            puts("Failed to change mode");
        }
        return 1;
    }

    if (verbose) {
        printf("mode of '%s' changed to %04o\n", path, (unsigned int)(target_mode & 07777));
    }

    if (recursive && S_ISDIR(st.st_mode)) {
        DIR *dir = opendir(path);
        if (dir) {
            struct dirent *ent;
            char subpath[256];
            while ((ent = readdir(dir)) != NULL) {
                if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) {
                    continue;
                }
                size_t plen = strlen(path);
                int has_slash = (plen > 0 && path[plen - 1] == '/');
                snprintf(subpath, sizeof(subpath), "%s%s%s", path, has_slash ? "" : "/",
                         ent->d_name);
                chmod_path(subpath, mode_str, is_octal, octal_val, recursive, verbose);
            }
            closedir(dir);
        }
    }

    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        print_usage();
        return 1;
    }

    int recursive = 0;
    int verbose = 0;
    const char *mode_str = NULL;
    int file_idx = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-R") == 0 || strcmp(argv[i], "--recursive") == 0) {
            recursive = 1;
        } else if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (!mode_str) {
            mode_str = argv[i];
            file_idx = i + 1;
            break;
        }
    }

    if (!mode_str || file_idx >= argc) {
        print_usage();
        return 1;
    }

    mode_t octal_val = 0;
    int is_octal = (parse_octal_mode(mode_str, &octal_val) == 0);

    int errors = 0;
    for (int i = file_idx; i < argc; i++) {
        if (chmod_path(argv[i], mode_str, is_octal, octal_val, recursive, verbose) != 0) {
            errors++;
        }
    }

    return (errors > 0) ? 1 : 0;
}
