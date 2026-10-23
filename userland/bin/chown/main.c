/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 File Ownership Utility (chown)
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
    puts("Usage: chown [OPTION]... [OWNER][:[GROUP]] FILE...");
    puts("");
    puts("Description:");
    puts("  Change the owner and/or group of each FILE to OWNER and/or GROUP.");
    puts("");
    puts("Options:");
    puts("  -R, --recursive  operate on files and directories recursively");
    puts("  -v, --verbose    output a diagnostic for every file processed");
    puts("  -h, --help       display this help message and exit");
    puts("");
    puts("Examples:");
    puts("  chown root:root /tmp/file");
    puts("  chown 1000:1000 /tmp/file");
    puts("  chown 1000 /tmp/file");
    puts("  chown :1000 /tmp/file");
}

static int parse_id(const char *str, uint32_t *id_out) {
    if (!str || *str == '\0') {
        return -1;
    }
    if (strcmp(str, "root") == 0) {
        *id_out = 0;
        return 0;
    }
    char *endptr;
    long val = strtol(str, &endptr, 10);
    if (*endptr != '\0' || val < 0) {
        return -1;
    }
    *id_out = (uint32_t)val;
    return 0;
}

static int parse_owner_group(const char *spec, uid_t *uid_out, gid_t *gid_out) {
    *uid_out = (uid_t)-1;
    *gid_out = (gid_t)-1;

    char buf[128];
    strncpy(buf, spec, sizeof(buf) - 1);
    buf[sizeof(buf) - 1] = '\0';

    char *colon = strchr(buf, ':');
    if (!colon) {
        // Only owner specified
        uint32_t uid_val;
        if (parse_id(buf, &uid_val) != 0) {
            return -1;
        }
        *uid_out = (uid_t)uid_val;
        return 0;
    }

    *colon = '\0';
    char *owner_part = buf;
    char *group_part = colon + 1;

    if (*owner_part != '\0') {
        uint32_t uid_val;
        if (parse_id(owner_part, &uid_val) != 0) {
            return -1;
        }
        *uid_out = (uid_t)uid_val;
    }

    if (*group_part != '\0') {
        uint32_t gid_val;
        if (parse_id(group_part, &gid_val) != 0) {
            return -1;
        }
        *gid_out = (gid_t)gid_val;
    }

    return 0;
}

static int chown_path(const char *path, uid_t uid, gid_t gid, int recursive, int verbose) {
    struct stat st;
    if (stat(path, &st) != 0) {
        printf("chown: cannot access '%s': ", path);
        if (errno == ENOENT) {
            puts("No such file or directory");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else {
            puts("Error accessing file");
        }
        return 1;
    }

    if (chown(path, uid, gid) != 0) {
        printf("chown: changing ownership of '%s': ", path);
        if (errno == EPERM) {
            puts("Operation not permitted");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else {
            puts("Failed to change ownership");
        }
        return 1;
    }

    if (verbose) {
        printf("ownership of '%s' changed to %u:%u\n", path,
               (unsigned int)(uid == (uid_t)-1 ? st.st_uid : uid),
               (unsigned int)(gid == (gid_t)-1 ? st.st_gid : gid));
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
                chown_path(subpath, uid, gid, recursive, verbose);
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
    const char *spec_str = NULL;
    int file_idx = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-R") == 0 || strcmp(argv[i], "--recursive") == 0) {
            recursive = 1;
        } else if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (!spec_str) {
            spec_str = argv[i];
            file_idx = i + 1;
            break;
        }
    }

    if (!spec_str || file_idx >= argc) {
        print_usage();
        return 1;
    }

    uid_t target_uid;
    gid_t target_gid;
    if (parse_owner_group(spec_str, &target_uid, &target_gid) != 0) {
        printf("chown: invalid user / group: '%s'\n", spec_str);
        return 1;
    }

    int errors = 0;
    for (int i = file_idx; i < argc; i++) {
        if (chown_path(argv[i], target_uid, target_gid, recursive, verbose) != 0) {
            errors++;
        }
    }

    return (errors > 0) ? 1 : 0;
}
