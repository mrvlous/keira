/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "preproc.h"

#include "common.h"

#include <syscall.h>

static struct Macro macro_table[MAX_MACROS];
static int macro_count = 0;

static int if_stack[MAX_IF_DEPTH];
static int if_taken[MAX_IF_DEPTH];
static int if_depth = 0;

static int include_depth = 0;
static char inc_buffers[MAX_INCLUDE_DEPTH][16384];

void init_preprocessor(void) {
    macro_count = 0;
    if_depth = 0;
    include_depth = 0;
    k_memset((char *)macro_table, 0, sizeof(macro_table));

    define_macro("__KEIRA__", "1");
#if defined(__i386__) || defined(__i686__)
    define_macro("__i386__", "1");
#else
    define_macro("__x86_64__", "1");
#endif
    define_macro("NULL", "0");
}

int define_macro(const char *name, const char *val) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            if (val && val[0]) {
                macro_table[i].has_val = 1;
                k_strcpy(macro_table[i].val, val);
            } else {
                macro_table[i].has_val = 0;
                macro_table[i].val[0] = '\0';
            }
            return 0;
        }
    }
    if (macro_count < MAX_MACROS) {
        k_strcpy(macro_table[macro_count].name, name);
        if (val && val[0]) {
            macro_table[macro_count].has_val = 1;
            k_strcpy(macro_table[macro_count].val, val);
        } else {
            macro_table[macro_count].has_val = 0;
            macro_table[macro_count].val[0] = '\0';
        }
        macro_table[macro_count].is_defined = 1;
        macro_count++;
        return 0;
    }
    return -1;
}

int undef_macro(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            macro_table[i].is_defined = 0;
            macro_table[i].has_val = 0;
            return 0;
        }
    }
    return -1;
}

int is_macro_defined(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            return 1;
        }
    }
    return 0;
}

const char *get_macro_val(const char *name) {
    int i;
    for (i = 0; i < macro_count; i++) {
        if (macro_table[i].is_defined && k_strcmp(macro_table[i].name, name) == 0) {
            if (macro_table[i].has_val) {
                return macro_table[i].val;
            }
            return "";
        }
    }
    return 0;
}

static int is_ident_char(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_';
}

static int is_ident_start(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

static void get_dir_of_path(const char *path, char *out_dir) {
    int len = k_strlen(path);
    int last_slash = -1;
    int i;
    for (i = 0; i < len; i++) {
        if (path[i] == '/') {
            last_slash = i;
        }
    }
    if (last_slash >= 0) {
        k_memcpy(out_dir, path, last_slash);
        out_dir[last_slash] = '\0';
    } else {
        out_dir[0] = '.';
        out_dir[1] = '\0';
    }
}

static int is_current_branch_active(void) {
    int i;
    for (i = 0; i < if_depth; i++) {
        if (!if_stack[i]) {
            return 0;
        }
    }
    return 1;
}

int preprocess_source(const char *input, char *output, int max_out, const char *cur_dir) {
    const char *p = input;
    int out_idx = 0;

    while (*p && out_idx < max_out - 1) {
        const char *line_start = p;
        while (*p && *p != '\n') {
            p++;
        }
        int line_len = (int)(p - line_start);
        if (*p == '\n') {
            p++;
        }

        const char *s = line_start;
        const char *line_end = line_start + line_len;
        while (s < line_end && (*s == ' ' || *s == '\t' || *s == '\r')) {
            s++;
        }

        if (s < line_end && *s == '#') {
            s++;
            while (s < line_end && (*s == ' ' || *s == '\t')) {
                s++;
            }

            char directive[32];
            int dlen = 0;
            while (s < line_end && *s >= 'a' && *s <= 'z' && dlen < 31) {
                directive[dlen++] = *s++;
            }
            directive[dlen] = '\0';

            while (s < line_end && (*s == ' ' || *s == '\t')) {
                s++;
            }

            if (k_strcmp(directive, "ifdef") == 0) {
                char name[MAX_MACRO_NAME];
                int nlen = 0;
                while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                    name[nlen++] = *s++;
                }
                name[nlen] = '\0';

                int parent_active = is_current_branch_active();
                int cond = is_macro_defined(name);
                if (if_depth < MAX_IF_DEPTH) {
                    if_stack[if_depth] = parent_active ? cond : 0;
                    if_taken[if_depth] = if_stack[if_depth];
                    if_depth++;
                }
                continue;
            } else if (k_strcmp(directive, "ifndef") == 0) {
                char name[MAX_MACRO_NAME];
                int nlen = 0;
                while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                    name[nlen++] = *s++;
                }
                name[nlen] = '\0';

                int parent_active = is_current_branch_active();
                int cond = !is_macro_defined(name);
                if (if_depth < MAX_IF_DEPTH) {
                    if_stack[if_depth] = parent_active ? cond : 0;
                    if_taken[if_depth] = if_stack[if_depth];
                    if_depth++;
                }
                continue;
            } else if (k_strcmp(directive, "if") == 0) {
                int parent_active = is_current_branch_active();
                int cond = 0;

                int negate = 0;
                if (*s == '!') {
                    negate = 1;
                    s++;
                    while (s < line_end && (*s == ' ' || *s == '\t')) {
                        s++;
                    }
                }

                if (k_strncmp(s, "defined", 7) == 0) {
                    s += 7;
                    while (s < line_end && (*s == ' ' || *s == '\t' || *s == '(')) {
                        s++;
                    }
                    char name[MAX_MACRO_NAME];
                    int nlen = 0;
                    while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                        name[nlen++] = *s++;
                    }
                    name[nlen] = '\0';
                    cond = is_macro_defined(name);
                } else if (*s == '1') {
                    cond = 1;
                } else if (*s == '0') {
                    cond = 0;
                } else if (is_ident_start(*s)) {
                    char name[MAX_MACRO_NAME];
                    int nlen = 0;
                    while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                        name[nlen++] = *s++;
                    }
                    name[nlen] = '\0';
                    cond = is_macro_defined(name);
                }

                if (negate) {
                    cond = !cond;
                }

                if (if_depth < MAX_IF_DEPTH) {
                    if_stack[if_depth] = parent_active ? cond : 0;
                    if_taken[if_depth] = if_stack[if_depth];
                    if_depth++;
                }
                continue;
            } else if (k_strcmp(directive, "else") == 0) {
                if (if_depth > 0) {
                    int parent_active = 1;
                    int k;
                    for (k = 0; k < if_depth - 1; k++) {
                        if (!if_stack[k]) {
                            parent_active = 0;
                            break;
                        }
                    }
                    if (parent_active && !if_taken[if_depth - 1]) {
                        if_stack[if_depth - 1] = 1;
                        if_taken[if_depth - 1] = 1;
                    } else {
                        if_stack[if_depth - 1] = 0;
                    }
                }
                continue;
            } else if (k_strcmp(directive, "endif") == 0) {
                if (if_depth > 0) {
                    if_depth--;
                }
                continue;
            }

            if (!is_current_branch_active()) {
                continue;
            }

            if (k_strcmp(directive, "define") == 0) {
                char name[MAX_MACRO_NAME];
                int nlen = 0;
                while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                    name[nlen++] = *s++;
                }
                name[nlen] = '\0';

                while (s < line_end && (*s == ' ' || *s == '\t')) {
                    s++;
                }

                char val[MAX_MACRO_VAL];
                int vlen = 0;
                while (s < line_end && *s != '\r' && *s != '\n' && vlen < MAX_MACRO_VAL - 1) {
                    val[vlen++] = *s++;
                }
                while (vlen > 0 && (val[vlen - 1] == ' ' || val[vlen - 1] == '\t')) {
                    vlen--;
                }
                val[vlen] = '\0';

                define_macro(name, vlen > 0 ? val : "");
                continue;
            } else if (k_strcmp(directive, "undef") == 0) {
                char name[MAX_MACRO_NAME];
                int nlen = 0;
                while (s < line_end && is_ident_char(*s) && nlen < MAX_MACRO_NAME - 1) {
                    name[nlen++] = *s++;
                }
                name[nlen] = '\0';
                undef_macro(name);
                continue;
            } else if (k_strcmp(directive, "include") == 0) {
                char target[128];
                int tlen = 0;
                char term = ' ';

                if (*s == '<') {
                    term = '>';
                    s++;
                } else if (*s == '"') {
                    term = '"';
                    s++;
                }

                while (s < line_end && *s != term && tlen < 127) {
                    target[tlen++] = *s++;
                }
                target[tlen] = '\0';

                if (include_depth >= MAX_INCLUDE_DEPTH) {
                    error_msg("Maximum #include recursion depth reached");
                    continue;
                }

                char full_path[160];
                int in_fd = -1;

                if (term == '"') {
                    int c_len = k_strlen(cur_dir);
                    if (c_len > 0 && cur_dir[0] != '.') {
                        k_strcpy(full_path, cur_dir);
                        if (full_path[c_len - 1] != '/') {
                            full_path[c_len] = '/';
                            full_path[c_len + 1] = '\0';
                        }
                        int fl = k_strlen(full_path);
                        k_strcpy(full_path + fl, target);
                        in_fd = sys_open(full_path, 0, 0);
                    }
                }

                if (in_fd < 0) {
                    k_strcpy(full_path, "/system/include/");
                    k_strcpy(full_path + 16, target);
                    in_fd = sys_open(full_path, 0, 0);
                }

                if (in_fd < 0) {
                    k_strcpy(full_path, "/system/include/sys/");
                    k_strcpy(full_path + 20, target);
                    in_fd = sys_open(full_path, 0, 0);
                }

                if (in_fd < 0) {
                    k_strcpy(full_path, target);
                    in_fd = sys_open(full_path, 0, 0);
                }

                if (in_fd >= 0) {
                    char *inc_buf = inc_buffers[include_depth];
                    k_memset(inc_buf, 0, 16384);
                    int r = sys_read(in_fd, inc_buf, 16384 - 1);
                    sys_close(in_fd);

                    if (r > 0) {
                        char next_dir[128];
                        get_dir_of_path(full_path, next_dir);

                        include_depth++;
                        int sub_len = preprocess_source(inc_buf, output + out_idx,
                                                        max_out - out_idx, next_dir);
                        include_depth--;

                        if (sub_len > 0) {
                            out_idx += sub_len;
                        }
                    }
                }
                continue;
            }
            continue;
        }

        if (!is_current_branch_active()) {
            continue;
        }

        /* Active code line: expand object-like macros */
        const char *c = line_start;
        int in_str = 0;
        int in_chr = 0;

        while (c < line_end && out_idx < max_out - 1) {
            if (*c == '"' && !in_chr) {
                in_str = !in_str;
                output[out_idx++] = *c++;
                continue;
            }
            if (*c == '\'' && !in_str) {
                in_chr = !in_chr;
                output[out_idx++] = *c++;
                continue;
            }

            if (!in_str && !in_chr && is_ident_start(*c)) {
                char word[MAX_MACRO_NAME];
                int wlen = 0;
                while (c < line_end && is_ident_char(*c) && wlen < MAX_MACRO_NAME - 1) {
                    word[wlen++] = *c++;
                }
                word[wlen] = '\0';

                if (is_macro_defined(word)) {
                    const char *val = get_macro_val(word);
                    if (val && val[0]) {
                        int vlen = k_strlen(val);
                        if (out_idx + vlen < max_out - 1) {
                            k_memcpy(output + out_idx, val, vlen);
                            out_idx += vlen;
                        }
                    }
                } else {
                    if (out_idx + wlen < max_out - 1) {
                        k_memcpy(output + out_idx, word, wlen);
                        out_idx += wlen;
                    }
                }
            } else {
                output[out_idx++] = *c++;
            }
        }

        if (out_idx < max_out - 1) {
            output[out_idx++] = '\n';
        }
    }

    output[out_idx] = '\0';
    return out_idx;
}
