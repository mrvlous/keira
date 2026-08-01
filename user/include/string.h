/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_STRING_H
#define KEIRA_USER_LIB_STRING_H

#include <stddef.h>

/**
 * Keira User-Space String and Memory Operations Interface
 */

/**
 * strlen - Calculate length of null-terminated string buffer.
 * @s: Target string buffer.
 *
 * Return: Length in bytes excluding terminating null byte.
 */
unsigned long strlen(const char *s);

/**
 * memcpy - Copy memory region from source to destination buffer.
 * @dest: Pointer to destination memory buffer.
 * @src: Pointer to source memory buffer.
 * @n: Byte count to copy.
 *
 * Return: Pointer to destination memory buffer.
 */
void *memcpy(void *dest, const void *src, unsigned long n);

/**
 * memset - Fill memory region with a constant byte value.
 * @s: Pointer to target memory buffer.
 * @c: Constant byte value.
 * @n: Byte count to fill.
 *
 * Return: Pointer to target memory buffer.
 */
void *memset(void *s, int c, unsigned long n);

/**
 * strcmp - Lexicographically compare two null-terminated strings.
 * @s1: First string pointer.
 * @s2: Second string pointer.
 *
 * Return: Difference between first non-matching bytes, or 0 if equal.
 */
int strcmp(const char *s1, const char *s2);

/**
 * strcpy - Copy null-terminated string into destination buffer.
 * @dest: Destination string buffer pointer.
 * @src: Source null-terminated string pointer.
 *
 * Return: Pointer to destination string buffer.
 */
char *strcpy(char *dest, const char *src);

/**
 * strncpy - Copy up to n characters from source to destination buffer.
 * @dest: Destination string buffer pointer.
 * @src: Source null-terminated string pointer.
 * @n: Maximum byte count to copy.
 *
 * Return: Pointer to destination string buffer.
 */
char *strncpy(char *dest, const char *src, unsigned long n);

char *strchr(const char *s, int c);
char *strrchr(const char *s, int c);
char *strstr(const char *haystack, const char *needle);
char *strtok(char *str, const char *delim);

#endif /* KEIRA_USER_LIB_STRING_H */
