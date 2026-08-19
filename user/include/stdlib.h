/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_STDLIB_H
#define KEIRA_USER_LIB_STDLIB_H

#include <stddef.h>

/**
 * Keira User-Space Standard Utility Interface (stdlib)
 */

void *malloc(size_t size);
void free(void *ptr);
void *realloc(void *ptr, size_t size);

/**
 * getenv - Retrieve environment variable string by name.
 * @name: Null-terminated variable key string.
 *
 * Return: Value string pointer or NULL if key is not defined.
 */
char *getenv(const char *name);

/**
 * setenv - Set or overwrite environment variable value.
 * @name: Null-terminated variable key string.
 * @value: Null-terminated variable value string.
 * @overwrite: 1 to overwrite existing key, 0 to keep existing key.
 *
 * Return: 0 on success, negative on error.
 */
int setenv(const char *name, const char *value, int overwrite);

/**
 * http_get - Fetch HTTP URL resource payload into buffer.
 * @url: Target URL string.
 * @buf: Destination memory buffer pointer.
 * @max_len: Maximum capacity limit in bytes.
 *
 * Return: Length of response bytes received.
 */
int http_get(const char *url, void *buf, int max_len);

#endif /* KEIRA_USER_LIB_STDLIB_H */