/* SPDX-License-Identifier: GPL-2.0-only */

#ifndef _STDDEF_H
#define _STDDEF_H

#ifndef NULL
#define NULL ((void *)0)
#endif

#if defined(__x86_64__) || defined(__LP64__) || \
    (defined(__SIZEOF_POINTER__) && __SIZEOF_POINTER__ == 8)
typedef unsigned long long size_t;
typedef long long ptrdiff_t;
#else
typedef unsigned int size_t;
typedef int ptrdiff_t;
#endif

#ifndef __KCC__
typedef long double max_align_t;
#endif

#define offsetof(type, member) ((size_t)&(((type *)0)->member))

#endif /* _STDDEF_H */
