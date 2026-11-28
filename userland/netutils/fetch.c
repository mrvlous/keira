/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Ring 3 HTTP/HTTPS Fetch Utility (fetch / curl)
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

#include <arpa/inet.h>
#include <ctype.h>
#include <fcntl.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#define FETCH_CHUNK_SIZE 1024
#define FETCH_HEADER_MAX 2048
#define FETCH_MAX_REDIRECTS 4

static void print_usage(const char *prog) {
    printf("Usage: %s [OPTIONS] <URL>\n", prog);
    puts("");
    puts("Description:");
    puts("  Freestanding Ring 3 HTTP and HTTPS client utility.");
    puts("");
    puts("Options:");
    puts("  -i, --include   Include response headers in standard output");
    puts("  -I, --head      Show response headers and status line only");
    puts("  -k, --insecure  Allow TLS connections without strict hostname matching");
    puts("  -L, --location  Follow HTTP 3xx Location redirects");
    puts("  -o, -O <file>   Write payload to local file instead of standard output");
    puts("  -s, --silent    Silent mode; suppress diagnostic and error messages");
    puts("  -v, --verbose   Display connection, TLS and transfer telemetry on stderr");
    puts("  -h, --help      Display this help reference and exit");
}

static int parse_url(const char *url, char *host, size_t host_sz, uint16_t *port, int *use_tls,
                     char *path, size_t path_sz) {
    if (!url || !host || !port || !use_tls || !path) {
        return -1;
    }

    const char *p = url;
    *use_tls = 0;
    uint16_t default_port = 80;
    if (strncmp(p, "https://", 8) == 0) {
        p += 8;
        *use_tls = 1;
        default_port = 443;
    } else if (strncmp(p, "http://", 7) == 0) {
        p += 7;
    }

    const char *slash = strchr(p, '/');
    const char *question = strchr(p, '?');
    const char *colon = strchr(p, ':');

    const char *term = slash;
    if (question && (!term || question < term)) {
        term = question;
    }

    if (term && colon && colon > term) {
        colon = NULL;
    }

    size_t host_len = 0;
    if (colon) {
        host_len = (size_t)(colon - p);
        *port = (uint16_t)atoi(colon + 1);
    } else if (term) {
        host_len = (size_t)(term - p);
        *port = default_port;
    } else {
        host_len = strlen(p);
        *port = default_port;
    }

    if (host_len == 0) {
        return -1;
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
    } else if (question) {
        snprintf(path, path_sz, "/%s", question);
    } else {
        strncpy(path, "/", path_sz - 1);
        path[path_sz - 1] = '\0';
    }

    return 0;
}

static int ci_contains(const char *haystack, const char *needle) {
    size_t nlen = strlen(needle);
    if (nlen == 0) {
        return 1;
    }
    for (const char *h = haystack; *h; h++) {
        size_t i = 0;
        while (i < nlen && h[i] &&
               tolower((unsigned char)h[i]) == tolower((unsigned char)needle[i])) {
            i++;
        }
        if (i == nlen) {
            return 1;
        }
    }
    return 0;
}

static int parse_status_code(const char *hdr) {
    const char *sp = strchr(hdr, ' ');
    if (!sp) {
        return 0;
    }
    return atoi(sp + 1);
}

static int extract_location(const char *hdr, char *out_loc, size_t out_sz) {
    const char *p = hdr;
    while (p && *p) {
        const char *eol = strchr(p, '\n');
        size_t line_len = eol ? (size_t)(eol - p) : strlen(p);
        if (line_len > 9) {
            const char *prefix = "location:";
            int match = 1;
            for (int i = 0; i < 9; i++) {
                if (tolower((unsigned char)p[i]) != prefix[i]) {
                    match = 0;
                    break;
                }
            }
            if (match) {
                const char *val = p + 9;
                const char *line_end = p + line_len;
                while (val < line_end && (*val == ' ' || *val == '\t')) {
                    val++;
                }
                while (line_end > val && (line_end[-1] == '\r' || line_end[-1] == ' ')) {
                    line_end--;
                }
                size_t len = (size_t)(line_end - val);
                if (len >= out_sz) {
                    len = out_sz - 1;
                }
                memcpy(out_loc, val, len);
                out_loc[len] = '\0';
                return len > 0 ? 1 : 0;
            }
        }
        if (!eol) {
            break;
        }
        p = eol + 1;
    }
    return 0;
}

struct chunk_decoder {
    int state;
    size_t remaining;
    int skip_crlf;
    int done;
};

static ssize_t emit_chunked_bytes(struct chunk_decoder *dec, int out_fd, const char *data,
                                  size_t len) {
    if (!data || len == 0 || len > FETCH_CHUNK_SIZE) {
        return 0;
    }
    char out_buf[FETCH_CHUNK_SIZE];
    size_t out_len = 0;
    ssize_t emitted = 0;
    for (size_t i = 0; i < len && !dec->done; i++) {
        char c = data[i];
        if (dec->state == 0) {
            if (dec->skip_crlf) {
                if (c == '\r' || c == '\n') {
                    continue;
                }
                dec->skip_crlf = 0;
            }
            if (c >= '0' && c <= '9') {
                dec->remaining = (dec->remaining << 4) | (size_t)(c - '0');
            } else if (c >= 'a' && c <= 'f') {
                dec->remaining = (dec->remaining << 4) | (size_t)(c - 'a' + 10);
            } else if (c >= 'A' && c <= 'F') {
                dec->remaining = (dec->remaining << 4) | (size_t)(c - 'A' + 10);
            } else if (c == '\n') {
                if (dec->remaining == 0) {
                    dec->done = 1;
                    break;
                }
                dec->state = 1;
            }
        } else {
            out_buf[out_len++] = c;
            dec->remaining--;
            if (dec->remaining == 0) {
                dec->state = 0;
                dec->skip_crlf = 1;
            }
        }
    }
    if (out_len > 0) {
        write(out_fd, out_buf, out_len);
        emitted = (ssize_t)out_len;
    }
    return emitted;
}

static int perform_fetch(const char *prog, const char *initial_url, const char *out_file,
                         int include_headers, int head_only, int insecure, int follow_redirects,
                         int silent, int verbose) {
    char current_url[512];
    strncpy(current_url, initial_url, sizeof(current_url) - 1);
    current_url[sizeof(current_url) - 1] = '\0';

    for (int hop = 0; hop <= FETCH_MAX_REDIRECTS; hop++) {
        char host[128];
        char path[256];
        uint16_t port = 80;
        int use_tls = 0;
        if (parse_url(current_url, host, sizeof(host), &port, &use_tls, path, sizeof(path)) < 0) {
            if (!silent) {
                fprintf(stderr, "%s: invalid URL syntax: %s\n", prog, current_url);
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
            fprintf(stderr, "* Connecting to %s:%u (path: %s, %s)\n", host, (unsigned int)port,
                    path, use_tls ? "HTTPS/TLS 1.3" : "HTTP");
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

        if (use_tls) {
            if (verbose && !silent) {
                fprintf(stderr, "* Initiating TLS 1.3 handshake with SNI '%s'\n", host);
            }
            int tls_flags = insecure ? TLS_FLAG_INSECURE : 0;
            if (sys_tls_attach(sockfd, host, tls_flags) < 0) {
                if (!silent) {
                    fprintf(stderr, "%s: TLS 1.3 handshake failed for %s:%u\n", prog, host,
                            (unsigned int)port);
                }
                close(sockfd);
                return 1;
            }
            if (verbose && !silent) {
                fprintf(stderr,
                        "* TLS 1.3 encrypted channel active (TLS_AES_128_GCM_SHA256, X25519)\n");
            }
        }

        char req[512];
        int req_len = snprintf(req, sizeof(req),
                               "%s %s HTTP/1.1\r\n"
                               "Host: %s\r\n"
                               "User-Agent: curl/keira-7.88.1\r\n"
                               "Accept: */*\r\n"
                               "Accept-Encoding: identity\r\n"
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

        int out_fd = -1;
        char chunk[FETCH_CHUNK_SIZE];
        int headers_parsed = 0;
        int is_chunked = 0;
        int redirecting = 0;
        char next_url[512];
        char header_buf[FETCH_HEADER_MAX];
        size_t header_buf_len = 0;
        ssize_t total_payload_bytes = 0;
        struct chunk_decoder dec;
        memset(&dec, 0, sizeof(dec));

        while (1) {
            ssize_t n = read(sockfd, chunk, sizeof(chunk));
            if (n <= 0) {
                break;
            }

            if (!headers_parsed) {
                if (header_buf_len < sizeof(header_buf) - 1) {
                    size_t space = sizeof(header_buf) - 1 - header_buf_len;
                    size_t to_copy = (size_t)n < space ? (size_t)n : space;
                    memcpy(header_buf + header_buf_len, chunk, to_copy);
                    header_buf_len += to_copy;
                }
                if (header_buf_len < sizeof(header_buf)) {
                    header_buf[header_buf_len] = '\0';
                }

                char *delim = strstr(header_buf, "\r\n\r\n");
                if (!delim) {
                    delim = strstr(header_buf, "\n\n");
                }

                if (delim) {
                    headers_parsed = 1;
                    int delim_len = (delim[0] == '\r') ? 4 : 2;
                    size_t hdr_len = (size_t)(delim - header_buf) + (size_t)delim_len;
                    size_t body_in_header = header_buf_len - hdr_len;
                    char *body_ptr = delim + delim_len;

                    char saved = *delim;
                    *delim = '\0';
                    int status_code = parse_status_code(header_buf);
                    if (ci_contains(header_buf, "transfer-encoding:") &&
                        ci_contains(header_buf, "chunked")) {
                        is_chunked = 1;
                    }

                    if (follow_redirects && !head_only &&
                        (status_code == 301 || status_code == 302 || status_code == 307 ||
                         status_code == 308) &&
                        hop < FETCH_MAX_REDIRECTS &&
                        extract_location(header_buf, next_url, sizeof(next_url))) {
                        *delim = saved;
                        redirecting = 1;
                        if (verbose && !silent) {
                            fprintf(stderr, "* Following redirect (%d) -> %s\n", status_code,
                                    next_url);
                        }
                        break;
                    }

                    if (verbose && !silent) {
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
                        fprintf(stderr, "<\n");
                    }
                    *delim = saved;

                    out_fd = STDOUT_FILENO;
                    if (out_file) {
                        out_fd = open(out_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
                        if (out_fd < 0) {
                            if (!silent) {
                                fprintf(stderr, "%s: cannot open output file '%s' for writing\n",
                                        prog, out_file);
                            }
                            close(sockfd);
                            return 1;
                        }
                    }

                    if (include_headers || head_only) {
                        write(out_fd, header_buf, hdr_len);
                    }
                    if (head_only) {
                        break;
                    }
                    if (body_in_header > 0) {
                        if (is_chunked) {
                            total_payload_bytes +=
                                emit_chunked_bytes(&dec, out_fd, body_ptr, body_in_header);
                        } else {
                            write(out_fd, body_ptr, body_in_header);
                            total_payload_bytes += (ssize_t)body_in_header;
                        }
                    }
                } else if (header_buf_len >= sizeof(header_buf) - 1) {
                    headers_parsed = 1;
                    out_fd = out_file ? open(out_file, O_WRONLY | O_CREAT | O_TRUNC, 0644)
                                      : STDOUT_FILENO;
                    if (out_fd < 0) {
                        close(sockfd);
                        return 1;
                    }
                    if (include_headers) {
                        write(out_fd, header_buf, header_buf_len);
                    }
                }
                continue;
            }

            if (!head_only && out_fd >= 0) {
                if (is_chunked) {
                    total_payload_bytes += emit_chunked_bytes(&dec, out_fd, chunk, (size_t)n);
                    if (dec.done) {
                        break;
                    }
                } else {
                    write(out_fd, chunk, (size_t)n);
                    total_payload_bytes += n;
                }
            }
        }

        close(sockfd);
        if (out_file && out_fd >= 0) {
            close(out_fd);
        }

        if (redirecting) {
            if (strncmp(next_url, "http://", 7) == 0 || strncmp(next_url, "https://", 8) == 0) {
                strncpy(current_url, next_url, sizeof(current_url) - 1);
                current_url[sizeof(current_url) - 1] = '\0';
            } else if (next_url[0] == '/') {
                snprintf(current_url, sizeof(current_url), "%s://%s%s", use_tls ? "https" : "http",
                         host, next_url);
            }
            continue;
        }

        if (verbose && !silent) {
            fprintf(stderr, "* Received %ld payload bytes\n", (long)total_payload_bytes);
            fprintf(stderr, "* Connection closed\n");
        }
        return 0;
    }

    if (!silent) {
        fprintf(stderr, "%s: maximum redirect hops exceeded\n", prog);
    }
    return 1;
}

int main(int argc, char **argv) {
    const char *prog = argv[0] ? argv[0] : "fetch";
    const char *url = NULL;
    const char *out_file = NULL;
    int include_headers = 0;
    int head_only = 0;
    int insecure = 0;
    int follow_redirects = 0;
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
        } else if (strcmp(argv[i], "-k") == 0 || strcmp(argv[i], "--insecure") == 0) {
            insecure = 1;
        } else if (strcmp(argv[i], "-L") == 0 || strcmp(argv[i], "--location") == 0) {
            follow_redirects = 1;
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

    return perform_fetch(prog, url, out_file, include_headers, head_only, insecure,
                         follow_redirects, silent, verbose);
}
