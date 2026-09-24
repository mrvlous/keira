/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <malloc.h>
#include <stdlib.h>
#include <string.h>

char **environ = NULL;

char *getenv(const char *name) {
    if (!name || !environ)
        return NULL;
    size_t len = strlen(name);
    for (char **env = environ; *env != NULL; env++) {
        if (strncmp(*env, name, len) == 0 && (*env)[len] == '=') {
            return *env + len + 1;
        }
    }
    return NULL;
}

int setenv(const char *name, const char *value, int overwrite) {
    if (!name || *name == '\0' || strchr(name, '=') != NULL) {
        errno = EINVAL;
        return -1;
    }
    if (!value)
        value = "";

    char *existing = getenv(name);
    if (existing && !overwrite)
        return 0;

    size_t name_len = strlen(name);
    size_t val_len = strlen(value);
    char *new_entry = (char *)malloc(name_len + 1 + val_len + 1);
    if (!new_entry) {
        errno = ENOMEM;
        return -1;
    }
    memcpy(new_entry, name, name_len);
    new_entry[name_len] = '=';
    memcpy(new_entry + name_len + 1, value, val_len);
    new_entry[name_len + 1 + val_len] = '\0';

    if (existing) {
        for (char **env = environ; *env != NULL; env++) {
            if (strncmp(*env, name, name_len) == 0 && (*env)[name_len] == '=') {
                *env = new_entry;
                return 0;
            }
        }
    }

    size_t count = 0;
    if (environ) {
        while (environ[count] != NULL)
            count++;
    }

    char **new_environ = (char **)malloc((count + 2) * sizeof(char *));
    if (!new_environ) {
        free(new_entry);
        errno = ENOMEM;
        return -1;
    }
    for (size_t i = 0; i < count; i++) {
        new_environ[i] = environ[i];
    }
    new_environ[count] = new_entry;
    new_environ[count + 1] = NULL;
    environ = new_environ;
    return 0;
}

int unsetenv(const char *name) {
    if (!name || *name == '\0' || strchr(name, '=') != NULL) {
        errno = EINVAL;
        return -1;
    }
    if (!environ)
        return 0;

    size_t len = strlen(name);
    char **src = environ;
    char **dst = environ;
    while (*src) {
        if (strncmp(*src, name, len) == 0 && (*src)[len] == '=') {
            src++;
        } else {
            *dst++ = *src++;
        }
    }
    *dst = NULL;
    return 0;
}

int putenv(char *string) {
    if (!string || strchr(string, '=') == NULL) {
        errno = EINVAL;
        return -1;
    }
    char *eq = strchr(string, '=');
    size_t name_len = (size_t)(eq - string);
    for (char **env = environ; env && *env != NULL; env++) {
        if (strncmp(*env, string, name_len) == 0 && (*env)[name_len] == '=') {
            *env = string;
            return 0;
        }
    }

    size_t count = 0;
    if (environ) {
        while (environ[count] != NULL)
            count++;
    }
    char **new_environ = (char **)malloc((count + 2) * sizeof(char *));
    if (!new_environ) {
        errno = ENOMEM;
        return -1;
    }
    for (size_t i = 0; i < count; i++) {
        new_environ[i] = environ[i];
    }
    new_environ[count] = string;
    new_environ[count + 1] = NULL;
    environ = new_environ;
    return 0;
}
