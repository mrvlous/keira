/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 File Removal Utility (rm)
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
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: rm [OPTION]... FILE...");
    puts("");
    puts("Description:");
    puts("  Remove (unlink) the FILE(s).");
    puts("");
    puts("Options:");
    puts("  -f, --force        ignore nonexistent files and arguments, never prompt");
    puts("  -r, -R, --recursive remove directories and their contents recursively");
    puts("  -v, --verbose      explain what is being done");
    puts("  -h, --help         display this help message and exit");
}

static int remove_recursive(const char *path, int force, int verbose);

static int remove_item(const char *path, int recursive, int force, int verbose) {
    DIR *dir = opendir(path);
    if (dir) {
        if (!recursive) {
            closedir(dir);
            printf("rm: cannot remove '%s': Is a directory\n", path);
            return 1;
        }
        closedir(dir);
        return remove_recursive(path, force, verbose);
    }

    if (unlink(path) != 0) {
        if (force && errno == ENOENT) {
            return 0;
        }
        printf("rm: cannot remove '%s': ", path);
        if (errno == ENOENT) {
            puts("No such file or directory");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else if (errno == EISDIR) {
            puts("Is a directory");
        } else {
            puts("Cannot remove file");
        }
        return 1;
    }

    if (verbose) {
        printf("removed '%s'\n", path);
    }
    return 0;
}

static int remove_recursive(const char *path, int force, int verbose) {
    DIR *dir = opendir(path);
    if (!dir) {
        return remove_item(path, 0, force, verbose);
    }

    struct dirent *ent;
    int status = 0;
    while ((ent = readdir(dir)) != NULL) {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) {
            continue;
        }

        char subpath[256];
        size_t plen = strlen(path);
        size_t nlen = strlen(ent->d_name);
        if (plen + nlen + 2 >= sizeof(subpath)) {
            printf("rm: path too long: %s/%s\n", path, ent->d_name);
            status = 1;
            continue;
        }

        if (plen > 0 && path[plen - 1] == '/') {
            snprintf(subpath, sizeof(subpath), "%s%s", path, ent->d_name);
        } else {
            snprintf(subpath, sizeof(subpath), "%s/%s", path, ent->d_name);
        }

        if (ent->d_type == DT_DIR) {
            if (remove_recursive(subpath, force, verbose) != 0) {
                status = 1;
            }
        } else {
            if (remove_item(subpath, 0, force, verbose) != 0) {
                status = 1;
            }
        }
    }
    closedir(dir);

    if (rmdir(path) != 0) {
        if (!force || errno != ENOENT) {
            printf("rm: cannot remove directory '%s': ", path);
            if (errno == ENOTEMPTY) {
                puts("Directory not empty");
            } else if (errno == EACCES) {
                puts("Permission denied");
            } else {
                puts("Cannot remove directory");
            }
            return 1;
        }
    } else if (verbose) {
        printf("removed directory '%s'\n", path);
    }

    return status;
}

int main(int argc, char **argv) {
    int opt_force = 0;
    int opt_recursive = 0;
    int opt_verbose = 0;
    const char *files[32];
    int file_count = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-f") == 0 || strcmp(arg, "--force") == 0) {
            opt_force = 1;
        } else if (strcmp(arg, "-r") == 0 || strcmp(arg, "-R") == 0 ||
                   strcmp(arg, "--recursive") == 0) {
            opt_recursive = 1;
        } else if (strcmp(arg, "-v") == 0 || strcmp(arg, "--verbose") == 0) {
            opt_verbose = 1;
        } else if (strcmp(arg, "-rf") == 0 || strcmp(arg, "-fr") == 0) {
            opt_recursive = 1;
            opt_force = 1;
        } else if (strcmp(arg, "-rv") == 0 || strcmp(arg, "-vr") == 0) {
            opt_recursive = 1;
            opt_verbose = 1;
        } else if (strcmp(arg, "-rfv") == 0 || strcmp(arg, "-frv") == 0) {
            opt_recursive = 1;
            opt_force = 1;
            opt_verbose = 1;
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("rm: unrecognized option '%s'\n", arg);
            puts("Try 'rm --help' for more information.");
            return 1;
        } else {
            if (file_count < 32) {
                files[file_count++] = arg;
            }
        }
    }

    if (file_count == 0) {
        if (!opt_force) {
            puts("rm: missing operand");
            puts("Try 'rm --help' for more information.");
            return 1;
        }
        return 0;
    }

    int exit_status = 0;
    for (int i = 0; i < file_count; i++) {
        if (remove_item(files[i], opt_recursive, opt_force, opt_verbose) != 0) {
            exit_status = 1;
        }
    }

    return exit_status;
}
