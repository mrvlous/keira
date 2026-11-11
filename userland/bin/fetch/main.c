/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Ring 3 HTTP Fetch Utility (fetch)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 */

#include <arpa/inet.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#define FETCH_BUF_SIZE 4096

static void print_usage(void) {
    puts("Usage: fetch [OPTIONS] <URL>");
    puts("");
    puts("Description:");
    puts("  Freestanding Ring 3 HTTP client utility to retrieve web resources.");
    puts("");
    puts("Options:");
    puts("  -o, -O <file>  Write payload to local file instead of standard output");
    puts("  -I, --head     Show response headers and status line only");
    puts("  -v, --verbose  Display connection and payload transfer telemetry");
    puts("  -h, --help     Display this help reference and exit");
}

static int parse_url(const char *url, char *host, size_t host_sz, uint16_t *port, char *path,
                     size_t path_sz) {
    if (!url || !host || !port || !path) {
        return -1;
    }

    const char *p = url;
    if (strncmp(p, "http://", 7) == 0) {
        p += 7;
    } else if (strncmp(p, "https://", 8) == 0) {
        p += 8;
    }

    const char *slash = strchr(p, '/');
    const char *colon = strchr(p, ':');

    if (slash && colon && colon > slash) {
        colon = NULL;
    }

    size_t host_len = 0;
    if (colon) {
        host_len = (size_t)(colon - p);
        *port = (uint16_t)atoi(colon + 1);
    } else if (slash) {
        host_len = (size_t)(slash - p);
        *port = 80;
    } else {
        host_len = strlen(p);
        *port = 80;
    }

    if (host_len >= host_sz) {
        host_len = host_sz - 1;
    }
    memcpy(host, p, host_len);
    host[host_len] = '\0';

    if (slash) {
        size_t p_len = strlen(slash);
        if (p_len >= path_sz) {
            p_len = path_sz - 1;
        }
        memcpy(path, slash, p_len);
        path[p_len] = '\0';
    } else {
        strncpy(path, "/", path_sz - 1);
        path[path_sz - 1] = '\0';
    }

    return 0;
}

int main(int argc, char **argv) {
    const char *url = NULL;
    const char *out_file = NULL;
    int head_only = 0;
    int verbose = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (strcmp(argv[i], "-I") == 0 || strcmp(argv[i], "--head") == 0) {
            head_only = 1;
        } else if (strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "-O") == 0) {
            if (i + 1 < argc) {
                out_file = argv[++i];
            } else {
                fprintf(stderr, "fetch: option '%s' requires an output file argument\n", argv[i]);
                return 1;
            }
        } else if (argv[i][0] != '-') {
            url = argv[i];
        } else {
            fprintf(stderr, "fetch: unrecognized option '%s'\n", argv[i]);
            return 1;
        }
    }

    if (!url) {
        print_usage();
        return 1;
    }

    char host[128];
    char path[256];
    uint16_t port = 80;
    if (parse_url(url, host, sizeof(host), &port, path, sizeof(path)) < 0) {
        fprintf(stderr, "fetch: invalid URL syntax: %s\n", url);
        return 1;
    }

    if (verbose) {
        printf("[fetch] Connecting to %s:%u (path: %s)\n", host, (unsigned int)port, path);
    }

    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        fprintf(stderr, "fetch: failed to create BSD socket\n");
        return 1;
    }

    struct sockaddr_in server_addr;
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(port);

    if (!inet_aton(host, &server_addr.sin_addr)) {
        if (verbose) {
            printf("[fetch] Resolving host %s to gateway 10.0.2.2\n", host);
        }
        inet_aton("10.0.2.2", &server_addr.sin_addr);
    }

    if (connect(sockfd, (const struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        fprintf(stderr, "fetch: connection failed to %s:%u\n", host, (unsigned int)port);
        close(sockfd);
        return 1;
    }

    char req[512];
    int req_len = snprintf(req, sizeof(req),
                           "GET %s HTTP/1.1\r\n"
                           "Host: %s:%u\r\n"
                           "User-Agent: fetch/1.0\r\n"
                           "Accept: */*\r\n"
                           "Connection: close\r\n\r\n",
                           path, host, (unsigned int)port);

    if (verbose) {
        printf("[fetch] Sending HTTP/1.1 request (%d bytes)\n", req_len);
    }

    ssize_t sent = write(sockfd, req, (size_t)req_len);
    if (sent < 0 || sent != req_len) {
        fprintf(stderr, "fetch: failed to transmit HTTP request\n");
        close(sockfd);
        return 1;
    }

    char *buf = (char *)malloc(FETCH_BUF_SIZE);
    if (!buf) {
        fprintf(stderr, "fetch: memory allocation error\n");
        close(sockfd);
        return 1;
    }
    memset(buf, 0, FETCH_BUF_SIZE);

    ssize_t n = read(sockfd, buf, FETCH_BUF_SIZE - 1);
    close(sockfd);

    if (n <= 0 || (size_t)n >= FETCH_BUF_SIZE) {
        fprintf(stderr, "fetch: failed to retrieve '%s': network or host error\n", url);
        free(buf);
        return 1;
    }

    buf[n] = '\0';

    if (verbose) {
        printf("[fetch] Successfully received %ld bytes\n", (long)n);
    }

    if (head_only) {
        char *header_end = strstr(buf, "\r\n\r\n");
        if (header_end) {
            size_t head_len = (size_t)(header_end - buf) + 2;
            write(STDOUT_FILENO, buf, head_len);
            putchar('\n');
        } else {
            char *line_end = strchr(buf, '\n');
            if (line_end) {
                size_t line_len = (size_t)(line_end - buf) + 1;
                write(STDOUT_FILENO, buf, line_len);
            } else {
                puts(buf);
            }
        }
        free(buf);
        return 0;
    }

    if (out_file) {
        int fd = open(out_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (fd < 0) {
            fprintf(stderr, "fetch: cannot open output file '%s' for writing\n", out_file);
            free(buf);
            return 1;
        }

        char *body_start = strstr(buf, "\r\n\r\n");
        const char *data_to_write = buf;
        size_t bytes_to_write = (size_t)n;

        if (body_start) {
            body_start += 4;
            data_to_write = body_start;
            bytes_to_write = (size_t)(n - (body_start - buf));
        }

        ssize_t written = write(fd, data_to_write, bytes_to_write);
        close(fd);

        if (written < 0 || (size_t)written != bytes_to_write) {
            fprintf(stderr, "fetch: write failure to file '%s'\n", out_file);
            free(buf);
            return 1;
        }

        if (verbose) {
            printf("[fetch] Wrote %ld payload bytes to '%s'\n", (long)written, out_file);
        }
    } else {
        write(STDOUT_FILENO, buf, (size_t)n);
    }

    free(buf);
    return 0;
}
