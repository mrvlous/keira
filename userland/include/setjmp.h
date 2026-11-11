/* SPDX-License-Identifier: GPL-2.0-only */

#ifndef _SETJMP_H
#define _SETJMP_H

typedef unsigned long jmp_buf[8];

int setjmp(jmp_buf env);
void longjmp(jmp_buf env, int val);

#endif /* _SETJMP_H */
