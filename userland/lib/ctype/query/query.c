// SPDX-License-Identifier: GPL-2.0-only

#include <ctype.h>

int isdigit(int c) {
    return (c >= '0' && c <= '9');
}

int isalpha(int c) {
    return ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'));
}

int isalnum(int c) {
    return isalpha(c) || isdigit(c);
}

int isspace(int c) {
    return (c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v');
}

int isupper(int c) {
    return (c >= 'A' && c <= 'Z');
}

int islower(int c) {
    return (c >= 'a' && c <= 'z');
}

int isprint(int c) {
    return (c >= 0x20 && c <= 0x7E);
}

int isgraph(int c) {
    return (c > 0x20 && c <= 0x7E);
}

int iscntrl(int c) {
    return ((c >= 0 && c < 0x20) || c == 0x7F);
}

int ispunct(int c) {
    return isgraph(c) && !isalnum(c);
}

int isxdigit(int c) {
    return isdigit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}
