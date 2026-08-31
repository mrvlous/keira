/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void main(void) {
    printf("Keira ASCII Mandelbrot Fractal (Fixed-Point Math)\n\n");

    int width = 50;
    int height = 18;
    int max_iter = 20;

    int y = -9;
    while (y < 9) {
        int x = -25;
        while (x < 25) {
            int cr = (x * 300) / width;
            int ci = (y * 300) / height;
            int zr = 0;
            int zi = 0;
            int iter = 0;

            while (iter < max_iter && (zr * zr + zi * zi) <= (4 * 10000)) {
                int next_zr = (zr * zr - zi * zi) / 100 + cr;
                int next_zi = (2 * zr * zi) / 100 + ci;
                zr = next_zr;
                zi = next_zi;
                iter = iter + 1;
            }

            if (iter == max_iter) {
                print_char(35); /* '#' */
            } else if (iter > 12) {
                print_char(42); /* '*' */
            } else if (iter > 6) {
                print_char(46); /* '.' */
            } else {
                print_char(32); /* ' ' */
            }
            x = x + 1;
        }
        print_char(10); /* '\n' */
        y = y + 1;
    }
    printf("\n[OK] Fractal rendering complete.\n");
}
