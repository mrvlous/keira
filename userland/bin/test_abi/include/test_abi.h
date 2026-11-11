/* SPDX-License-Identifier: GPL-2.0-only */

#ifndef _TEST_ABI_H
#define _TEST_ABI_H

#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/io_uring.h>
#include <sys/mman.h>
#include <sys/random.h>
#include <sys/reboot.h>
#include <sys/resource.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

int run_test_abi_proc(void);
int run_test_abi_mem(void);
int run_test_abi_fs(void);
int run_test_abi_ipc(void);
int run_test_abi_signal(void);
int run_test_abi_stress(void);

#endif /* _TEST_ABI_H */
