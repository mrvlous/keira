/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Mount Utility (mount)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: mount [-t FSTYPE] [-o OPTIONS] [DEVICE] DIRECTORY");
    puts("       mount [-h]");
    puts("");
    puts("Description:");
    puts("  Mount a filesystem or display registered kernel mount points.");
    puts("");
    puts("Options:");
    puts("  -t, --type FSTYPE     specify filesystem type (fat, procfs, devfs, tmpfs, ext4)");
    puts("  -o, --options OPTS    comma-separated list of mount options (ro, rw, nosuid, remount)");
    puts("  -h, --help            display this help message and exit");
}

static int display_mounts(void) {
    int fd = open("/proc/mounts", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "mount: cannot open /proc/mounts\n");
        return 1;
    }

    char buf[2048];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        puts("No filesystems mounted.");
        return 0;
    }

    buf[n] = '\0';
    char *line = buf;
    while (*line) {
        char *end = strchr(line, '\n');
        if (end) {
            *end = '\0';
        }

        char src[64], tgt[128], fstype[32], opts[64];
        int dump = 0, pass = 0;
        if (sscanf(line, "%63s %127s %31s %63s %d %d", src, tgt, fstype, opts, &dump, &pass) >= 4) {
            printf("%s on %s type %s (%s)\n", src, tgt, fstype, opts);
        }

        if (!end) {
            break;
        }
        line = end + 1;
    }

    return 0;
}

int main(int argc, char **argv) {
    const char *fstype = "fat";
    const char *options = NULL;
    const char *source = NULL;
    const char *target = NULL;
    unsigned long flags = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-t") == 0 && i + 1 < argc) {
            fstype = argv[++i];
        } else if (strcmp(argv[i], "-o") == 0 && i + 1 < argc) {
            options = argv[++i];
        } else if (argv[i][0] == '-') {
            fprintf(stderr, "mount: unrecognized option '%s'\n", argv[i]);
            return 1;
        } else if (!source && i + 1 < argc) {
            source = argv[i];
        } else if (!target) {
            target = argv[i];
        }
    }

    // Bare mount: display table
    if (!source && !target) {
        return display_mounts();
    }

    // Single argument: target with default source
    if (source && !target) {
        target = source;
        source = "none";
    }

    if (!source) {
        source = "none";
    }

    if (options) {
        if (strstr(options, "ro")) {
            flags |= MS_RDONLY;
        }
        if (strstr(options, "nosuid")) {
            flags |= MS_NOSUID;
        }
        if (strstr(options, "nodev")) {
            flags |= MS_NODEV;
        }
        if (strstr(options, "noexec")) {
            flags |= MS_NOEXEC;
        }
        if (strstr(options, "remount")) {
            flags |= MS_REMOUNT;
        }
    }

    int res = mount(source, target, fstype, flags, NULL);
    if (res < 0) {
        fprintf(stderr, "mount: mounting %s on %s failed (error %d)\n", source, target, res);
        return 1;
    }

    printf("mount: successfully mounted %s on %s (type %s)\n", source, target, fstype);
    return 0;
}
