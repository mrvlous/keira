/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Directory Listing Utility (ls)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void format_mode(mode_t mode, char *str) {
    if (S_ISDIR(mode)) {
        str[0] = 'd';
    } else if (S_ISCHR(mode)) {
        str[0] = 'c';
    } else if (S_ISBLK(mode)) {
        str[0] = 'b';
    } else if (S_ISLNK(mode)) {
        str[0] = 'l';
    } else {
        str[0] = '-';
    }

    str[1] = (mode & S_IRUSR) ? 'r' : '-';
    str[2] = (mode & S_IWUSR) ? 'w' : '-';
    str[3] = (mode & S_IXUSR) ? 'x' : '-';
    str[4] = (mode & S_IRGRP) ? 'r' : '-';
    str[5] = (mode & S_IWGRP) ? 'w' : '-';
    str[6] = (mode & S_IXGRP) ? 'x' : '-';
    str[7] = (mode & S_IROTH) ? 'r' : '-';
    str[8] = (mode & S_IWOTH) ? 'w' : '-';
    str[9] = (mode & S_IXOTH) ? 'x' : '-';
    str[10] = '\0';
}

static int list_directory(const char *path, int show_all, int long_format) {
    struct stat file_st;
    if (stat(path, &file_st) == 0 && !S_ISDIR(file_st.st_mode)) {
        if (long_format) {
            char mode_str[11];
            format_mode(file_st.st_mode, mode_str);
            printf("%s %2u %4u %4u %8ld %s\n", mode_str, (unsigned)file_st.st_nlink,
                   (unsigned)file_st.st_uid, (unsigned)file_st.st_gid, (long)file_st.st_size, path);
        } else {
            printf("%s\n", path);
        }
        return 0;
    }

    DIR *dir = opendir(path);
    if (!dir) {
        printf("ls: cannot access '%s': No such file or directory\n", path);
        return 1;
    }

    struct dirent *ent;
    int entry_count = 0;

    while ((ent = readdir(dir)) != NULL) {
        if (!show_all && ent->d_name[0] == '.') {
            continue;
        }

        entry_count++;
        if (long_format) {
            char full_path[256];
            size_t plen = strlen(path);
            int has_slash = (plen > 0 && path[plen - 1] == '/');
            snprintf(full_path, sizeof(full_path), "%s%s%s", path, has_slash ? "" : "/",
                     ent->d_name);

            struct stat st;
            if (stat(full_path, &st) == 0) {
                char mode_str[11];
                format_mode(st.st_mode, mode_str);
                printf("%s %2u %4u %4u %8ld %s\n", mode_str, (unsigned)st.st_nlink,
                       (unsigned)st.st_uid, (unsigned)st.st_gid, (long)st.st_size, ent->d_name);
            } else {
                if (ent->d_type == DT_DIR) {
                    printf("drwxr-xr-x  [dir]   %s\n", ent->d_name);
                } else {
                    printf("-rw-r--r--  [file]  %s\n", ent->d_name);
                }
            }
        } else {
            printf("%s  ", ent->d_name);
        }
    }

    if (!long_format && entry_count > 0) {
        putchar('\n');
    }

    closedir(dir);
    return 0;
}

int main(int argc, char **argv) {
    int show_all = 0;
    int long_format = 0;
    const char *paths[32];
    int path_count = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            puts("Usage: ls [OPTIONS] [PATH...]");
            puts("");
            puts("Description:");
            puts("  List directory contents and metadata.");
            puts("");
            puts("Options:");
            puts("  -a, --all      Do not ignore entries starting with .");
            puts("  -l             Use long listing format");
            puts("  -h, --help     Display this help message and exit");
            return 0;
        }

        if (strcmp(argv[i], "-a") == 0 || strcmp(argv[i], "--all") == 0) {
            show_all = 1;
        } else if (strcmp(argv[i], "-l") == 0) {
            long_format = 1;
        } else if (strcmp(argv[i], "-la") == 0 || strcmp(argv[i], "-al") == 0) {
            show_all = 1;
            long_format = 1;
        } else if (argv[i][0] != '-') {
            if (path_count < 32) {
                paths[path_count++] = argv[i];
            }
        }
    }

    if (path_count == 0) {
        paths[0] = ".";
        path_count = 1;
    }

    int exit_status = 0;
    for (int i = 0; i < path_count; i++) {
        if (path_count > 1) {
            printf("%s:\n", paths[i]);
        }
        if (list_directory(paths[i], show_all, long_format) != 0) {
            exit_status = 1;
        }
        if (path_count > 1 && i < path_count - 1) {
            putchar('\n');
        }
    }

    return exit_status;
}
