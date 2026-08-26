/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _STDINT_H
#define _STDINT_H

typedef signed char int8_t;
typedef short int16_t;
typedef int int32_t;
typedef long long int64_t;

typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#if defined(__x86_64__) || defined(__LP64__) || (defined(__SIZEOF_POINTER__) && __SIZEOF_POINTER__ == 8)
typedef long long intptr_t;
typedef unsigned long long uintptr_t;
#else
typedef int intptr_t;
typedef unsigned int uintptr_t;
#endif

typedef int64_t intmax_t;
typedef uint64_t uintmax_t;

#define INT8_MIN (-128)
#define INT8_MAX (127)
#define UINT8_MAX (255)

#define INT16_MIN (-32768)
#define INT16_MAX (32767)
#define UINT16_MAX (65535)

#define INT32_MIN (-2147483647 - 1)
#define INT32_MAX (2147483647)
#define UINT32_MAX (4294967295U)

#define INT64_MIN (-9223372036854775807LL - 1)
#define INT64_MAX (9223372036854775807LL)
#define UINT64_MAX (18446744073709551615ULL)

#endif /* _STDINT_H */
