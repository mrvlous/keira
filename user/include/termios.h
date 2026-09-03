/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _TERMIOS_H
#define _TERMIOS_H

#include <stdint.h>
#include <sys/types.h>

typedef uint32_t tcflag_t;
typedef uint8_t cc_t;
typedef uint32_t speed_t;

#define NCCS 32

struct termios {
    tcflag_t c_iflag; /* Input modes */
    tcflag_t c_oflag; /* Output modes */
    tcflag_t c_cflag; /* Control modes */
    tcflag_t c_lflag; /* Local modes */
    cc_t c_line;      /* Line discipline */
    cc_t c_cc[NCCS];  /* Special characters */
    speed_t c_ispeed; /* Input baud rate */
    speed_t c_ospeed; /* Output baud rate */
};

/* Terminal Control Requests (ioctl) */
#define TCGETS 0x5401
#define TCSETS 0x5402
#define TCSETSW 0x5403
#define TCSETSF 0x5404

/* Actions for tcsetattr */
#define TCSANOW 0
#define TCSADRAIN 1
#define TCSAFLUSH 2

/* Local modes (c_lflag) */
#define ISIG 0000001
#define ICANON 0000002
#define ECHO 0000010
#define ECHOE 0000020
#define ECHOK 0000040
#define ECHONL 0000100
#define NOFLSH 0000200
#define TOSTOP 0000400
#define IEXTEN 0100000

/* Input modes (c_iflag) */
#define IGNBRK 0000001
#define BRKINT 0000002
#define IGNPAR 0000004
#define PARMRK 0000010
#define INPCK 0000020
#define ISTRIP 0000040
#define INLCR 0000100
#define IGNCR 0000200
#define ICRNL 0000400
#define IXON 0002000
#define IXOFF 0010000

/* Output modes (c_oflag) */
#define OPOST 0000001
#define ONLCR 0000004

/* Control characters (c_cc indices) */
#define VINTR 0
#define VQUIT 1
#define VERASE 2
#define VKILL 3
#define VEOF 4
#define VTIME 5
#define VMIN 6

int tcgetattr(int fd, struct termios *termios_p);
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p);

#endif /* _TERMIOS_H */
