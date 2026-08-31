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

void main(void) {
    printf("Keira ASCII Mandelbrot Fractal (Fixed-Point Math)\n\n");

    int width = 60;
    int height = 20;
    int max_iter = 20;

    int y = -10;
    while (y < 10) {
        int x = -30;
        while (x < 30) {
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
                iter++;
            }

            if (iter == max_iter) {
                putchar('#');
            } else if (iter > 12) {
                putchar('*');
            } else if (iter > 6) {
                putchar('.');
            } else {
                putchar(' ');
            }
            x++;
        }
        putchar('\n');
        y++;
    }
    printf("\n[OK] Fractal rendering complete.\n");
}
