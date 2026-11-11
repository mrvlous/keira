// SPDX-License-Identifier: GPL-2.0-only

#include <ctype.h>

int tolower(int c) {
    if (isupper(c))
        return c + ('a' - 'A');
    return c;
}

int toupper(int c) {
    if (islower(c))
        return c - ('a' - 'A');
    return c;
}
