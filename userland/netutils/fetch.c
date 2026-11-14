// SPDX-License-Identifier: GPL-2.0-only
/*
 * Keira Kernel - Freestanding Ring 3 HTTP Fetch Utility (fetch / curl)
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

#include <arpa/inet.h>
#include <fcntl.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#define FETCH_CHUNK_SIZE 1024
#define FETCH_HEADER_MAX 2048

static void print_usage(const char *prog) {
    printf("Usage: %s [OPTIONS] <URL>\n", prog);
    puts("");
    puts("Description:");
    puts("  Freestanding Ring 3 HTTP client utility to retrieve web resources.");
    puts("");
    puts("Options:");
    puts("  -i, --include  Include HTTP response headers in standard output");
    puts("  -I, --head     Show response headers and status line only");
    puts("  -o, -O <file>  Write payload to local file instead of standard output");
    puts("  -s, --silent   Silent mode; suppress diagnostic and error messages");
    puts("  -v, --verbose  Display connection and payload transfer telemetry on stderr");
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
    const char *prog = argv[0] ? argv[0] : "fetch";
    const char *url = NULL;
    const char *out_file = NULL;
    int include_headers = 0;
    int head_only = 0;
    int silent = 0;
    int verbose = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage(prog);
            return 0;
        }
        if (strcmp(argv[i], "-i") == 0 || strcmp(argv[i], "--include") == 0) {
            include_headers = 1;
        } else if (strcmp(argv[i], "-I") == 0 || strcmp(argv[i], "--head") == 0) {
            head_only = 1;
        } else if (strcmp(argv[i], "-s") == 0 || strcmp(argv[i], "--silent") == 0) {
            silent = 1;
        } else if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "-O") == 0) {
            if (i + 1 < argc) {
                out_file = argv[++i];
            } else {
                if (!silent) {
                    fprintf(stderr, "%s: option '%s' requires an output file argument\n", prog,
                            argv[i]);
                }
                return 1;
            }
        } else if (argv[i][0] != '-') {
            url = argv[i];
        } else {
            if (!silent) {
                fprintf(stderr, "%s: unrecognized option '%s'\n", prog, argv[i]);
            }
            return 1;
        }
    }

    if (!url) {
        print_usage(prog);
        return 1;
    }

    char host[128];
    char path[256];
    uint16_t port = 80;
    if (parse_url(url, host, sizeof(host), &port, path, sizeof(path)) < 0) {
        if (!silent) {
            fprintf(stderr, "%s: invalid URL syntax: %s\n", prog, url);
        }
        return 1;
    }

    struct hostent *he = gethostbyname(host);
    if (!he || !he->h_addr_list || !he->h_addr_list[0]) {
        if (!silent) {
            fprintf(stderr, "%s: could not resolve host '%s'\n", prog, host);
        }
        return 1;
    }

    struct sockaddr_in server_addr;
    memset(&server_addr, 0, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(port);
    memcpy(&server_addr.sin_addr, he->h_addr_list[0], sizeof(server_addr.sin_addr));

    if (verbose && !silent) {
        fprintf(stderr, "* Resolved %s to %s\n", host, inet_ntoa(server_addr.sin_addr));
        fprintf(stderr, "* Connecting to %s:%u (path: %s)\n", host, (unsigned int)port, path);
    }

    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        if (!silent) {
            fprintf(stderr, "%s: failed to create BSD socket\n", prog);
        }
        return 1;
    }

    if (connect(sockfd, (const struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        if (!silent) {
            fprintf(stderr, "%s: connection failed to %s:%u\n", prog, host, (unsigned int)port);
        }
        close(sockfd);
        return 1;
    }

    char req[512];
    int req_len = snprintf(req, sizeof(req),
                           "%s %s HTTP/1.1\r\n"
                           "Host: %s\r\n"
                           "User-Agent: curl/keira-7.88.1\r\n"
                           "Accept: */*\r\n"
                           "Connection: close\r\n\r\n",
                           head_only ? "HEAD" : "GET", path, host);

    if (verbose && !silent) {
        fprintf(stderr, "> %s %s HTTP/1.1\n", head_only ? "HEAD" : "GET", path);
        fprintf(stderr, "> Host: %s\n", host);
        fprintf(stderr, "> User-Agent: curl/keira-7.88.1\n");
        fprintf(stderr, "> Accept: */*\n");
        fprintf(stderr, "> Connection: close\n>\n");
    }

    ssize_t sent = write(sockfd, req, (size_t)req_len);
    if (sent < 0 || sent != req_len) {
        if (!silent) {
            fprintf(stderr, "%s: failed to transmit HTTP request\n", prog);
        }
        close(sockfd);
        return 1;
    }

    int out_fd = STDOUT_FILENO;
    if (out_file) {
        out_fd = open(out_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (out_fd < 0) {
            if (!silent) {
                fprintf(stderr, "%s: cannot open output file '%s' for writing\n", prog, out_file);
            }
            close(sockfd);
            return 1;
        }
    }

    char chunk[FETCH_CHUNK_SIZE];
    int headers_parsed = 0;
    char header_buf[FETCH_HEADER_MAX];
    size_t header_buf_len = 0;
    ssize_t total_payload_bytes = 0;

    while (1) {
        ssize_t n = read(sockfd, chunk, sizeof(chunk));
        if (n <= 0) {
            break;
        }

        if (!headers_parsed) {
            size_t space = sizeof(header_buf) - 1 - header_buf_len;
            size_t to_copy = (size_t)n < space ? (size_t)n : space;
            memcpy(header_buf + header_buf_len, chunk, to_copy);
            header_buf_len += to_copy;
            header_buf[header_buf_len] = '\0';

            char *delim = strstr(header_buf, "\r\n\r\n");
            if (!delim) {
                delim = strstr(header_buf, "\n\n");
            }

            if (delim) {
                headers_parsed = 1;
                int delim_len = (delim[0] == '\r') ? 4 : 2;
                size_t hdr_len = (size_t)(delim - header_buf) + delim_len;
                size_t body_in_header = header_buf_len - hdr_len;
                char *body_ptr = delim + delim_len;

                if (verbose && !silent) {
                    char saved = *delim;
                    *delim = '\0';
                    char *line_p = header_buf;
                    while (line_p && *line_p) {
                        char *nl = strchr(line_p, '\n');
                        if (nl) {
                            *nl = '\0';
                            if (nl > line_p && *(nl - 1) == '\r') {
                                *(nl - 1) = '\0';
                            }
                            fprintf(stderr, "< %s\n", line_p);
                            line_p = nl + 1;
                        } else {
                            fprintf(stderr, "< %s\n", line_p);
                            break;
                        }
                    }
                    *delim = saved;
                    fprintf(stderr, "<\n");
                }

                if (include_headers || head_only) {
                    write(out_fd, header_buf, hdr_len);
                }

                if (head_only) {
                    break;
                }

                if (body_in_header > 0) {
                    write(out_fd, body_ptr, body_in_header);
                    total_payload_bytes += (ssize_t)body_in_header;
                }
            } else if (header_buf_len >= sizeof(header_buf) - 1) {
                headers_parsed = 1;
                if (include_headers) {
                    write(out_fd, header_buf, header_buf_len);
                }
            }
            continue;
        }

        if (!head_only) {
            write(out_fd, chunk, (size_t)n);
            total_payload_bytes += n;
        }
    }

    close(sockfd);
    if (out_file && out_fd >= 0) {
        close(out_fd);
    }

    if (verbose && !silent) {
        fprintf(stderr, "* Received %ld payload bytes\n", (long)total_payload_bytes);
        fprintf(stderr, "* Connection closed\n");
    }

    return 0;
}
