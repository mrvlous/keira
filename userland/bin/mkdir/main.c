/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Directory Creation Utility (mkdir)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static void print_usage(void) {
    puts("Usage: mkdir [OPTION]... DIRECTORY...");
    puts("");
    puts("Description:");
    puts("  Create the DIRECTORY(ies), if they do not already exist.");
    puts("");
    puts("Options:");
    puts("  -p, --parents  no error if existing, make parent directories as needed");
    puts("  -v, --verbose  print a message for each created directory");
    puts("  -h, --help     display this help message and exit");
}

static int make_parents(const char *path, int verbose) {
    char tmp[256];
    size_t len = strlen(path);
    if (len >= sizeof(tmp)) {
        printf("mkdir: path too long: %s\n", path);
        return 1;
    }
    memcpy(tmp, path, len + 1);

    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(tmp, 0755) != 0) {
                if (errno != EEXIST) {
                    printf("mkdir: cannot create directory '%s': ", tmp);
                    if (errno == EACCES) {
                        puts("Permission denied");
                    } else {
                        puts("No such file or directory");
                    }
                    return 1;
                }
            } else if (verbose) {
                printf("mkdir: created directory '%s'\n", tmp);
            }
            *p = '/';
        }
    }

    if (mkdir(tmp, 0755) != 0) {
        if (errno != EEXIST) {
            printf("mkdir: cannot create directory '%s': ", tmp);
            if (errno == EACCES) {
                puts("Permission denied");
            } else {
                puts("Cannot create directory");
            }
            return 1;
        }
    } else if (verbose) {
        printf("mkdir: created directory '%s'\n", tmp);
    }
    return 0;
}

static int make_single(const char *path, int verbose) {
    if (mkdir(path, 0755) != 0) {
        printf("mkdir: cannot create directory '%s': ", path);
        if (errno == EEXIST) {
            puts("File exists");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else if (errno == ENOENT) {
            puts("No such file or directory");
        } else {
            puts("Cannot create directory");
        }
        return 1;
    }
    if (verbose) {
        printf("mkdir: created directory '%s'\n", path);
    }
    return 0;
}

int main(int argc, char **argv) {
    int opt_parents = 0;
    int opt_verbose = 0;
    const char *dirs[32];
    int dir_count = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-p") == 0 || strcmp(arg, "--parents") == 0) {
            opt_parents = 1;
        } else if (strcmp(arg, "-v") == 0 || strcmp(arg, "--verbose") == 0) {
            opt_verbose = 1;
        } else if (strcmp(arg, "-pv") == 0 || strcmp(arg, "-vp") == 0) {
            opt_parents = 1;
            opt_verbose = 1;
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("mkdir: unrecognized option '%s'\n", arg);
            puts("Try 'mkdir --help' for more information.");
            return 1;
        } else {
            if (dir_count < 32) {
                dirs[dir_count++] = arg;
            }
        }
    }

    if (dir_count == 0) {
        puts("mkdir: missing operand");
        puts("Try 'mkdir --help' for more information.");
        return 1;
    }

    int exit_status = 0;
    for (int i = 0; i < dir_count; i++) {
        int ret =
            opt_parents ? make_parents(dirs[i], opt_verbose) : make_single(dirs[i], opt_verbose);
        if (ret != 0) {
            exit_status = 1;
        }
    }

    return exit_status;
}
