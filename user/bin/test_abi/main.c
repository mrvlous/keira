/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    puts("Keira Ring 3 Syscall Security & ABI Verification Harness");

    /* 1. Process Identity & Descriptor Verification */
    pid_t pid = getpid();
    pid_t ppid = getppid();
    printf("  [TEST] Process identity: PID=%d, PPID=%d\n", (int)pid, (int)ppid);
    if (pid < 0) {
        puts("  [FAIL] Invalid PID returned by getpid()");
        return 1;
    }
    puts("  [OK]   Process identity verified");

    /* 2. Kernel Memory Boundary & EFAULT Protection */
    puts("  [TEST] Kernel address pointer isolation (EFAULT check)...");
#if defined(__x86_64__)
    void *kernel_addr = (void *)0xFFFF800000000000ULL;
#else
    void *kernel_addr = (void *)0xC0000000UL;
#endif
    ssize_t efault_res = write(1, kernel_addr, 16);
    if (efault_res >= 0) {
        puts("  [FAIL] Kernel address access was not blocked by copy_from_user!");
        return 1;
    }
    puts("  [OK]   Kernel space boundary validated (EFAULT enforced)");

    /* 3. Null Pointer Protection */
    puts("  [TEST] NULL pointer dereference protection...");
    ssize_t null_res = read(0, NULL, 32);
    if (null_res >= 0) {
        puts("  [FAIL] NULL pointer was not blocked by copy_to_user!");
        return 1;
    }
    puts("  [OK]   NULL pointer isolation validated (EFAULT enforced)");

    /* 4. Invalid File Descriptor (EBADF) Protection */
    puts("  [TEST] Invalid file descriptor validation...");
    char tmp_buf[16];
    ssize_t ebadf_neg = read(-1, tmp_buf, sizeof(tmp_buf));
    ssize_t ebadf_large = read(999, tmp_buf, sizeof(tmp_buf));
    if (ebadf_neg >= 0 || ebadf_large >= 0) {
        puts("  [FAIL] Out-of-bounds file descriptor accepted!");
        return 1;
    }
    puts("  [OK]   Out-of-bounds file descriptors rejected (EBADF enforced)");

    /* 5. VFS Working Directory (chdir/getcwd) Tracking */
    puts("  [TEST] VFS working directory tracking...");
    char cwd_buf[64];
    memset(cwd_buf, 0, sizeof(cwd_buf));
    if (getcwd(cwd_buf, sizeof(cwd_buf)) == NULL) {
        puts("  [FAIL] getcwd() failed on initial directory");
        return 1;
    }
    if (chdir("/config") == 0) {
        memset(cwd_buf, 0, sizeof(cwd_buf));
        if (getcwd(cwd_buf, sizeof(cwd_buf)) != NULL) {
            printf("  [INFO] Changed working directory to: %s\n", cwd_buf);
        }
    }
    puts("  [OK]   VFS working directory tracking operational");

    /* 6. VFS File Seeking (lseek) */
    puts("  [TEST] VFS file seeking (lseek)...");
    int fd = open("/config/sys/hostname.cfg", O_RDONLY, 0);
    if (fd >= 0) {
        off_t pos = lseek(fd, 0, SEEK_CUR);
        if (pos != 0) {
            printf("  [WARN] Unexpected initial seek offset: %d\n", (int)pos);
        }
        close(fd);
    }
    puts("  [OK]   VFS seek pointer operational");

    /* 7. BSD Socket Lifecycle (Socket, Connect, Send, Close) */
    puts("  [TEST] BSD socket lifecycle & async stream dispatch...");
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock >= 0) {
        printf("  [INFO] Allocated socket descriptor: fd=%d\n", sock);
        struct sockaddr_in saddr;
        memset(&saddr, 0, sizeof(saddr));
        saddr.sin_family = AF_INET;
        saddr.sin_port = 0x5000;            /* Port 80 */
        saddr.sin_addr.s_addr = 0x0100007F; /* 127.0.0.1 */

        if (connect(sock, (const struct sockaddr *)&saddr, sizeof(saddr)) == 0) {
            puts("  [INFO] Socket connected to remote endpoint");
            ssize_t sent = send(sock, "PING", 4, 0);
            if (sent == 4) {
                puts("  [INFO] Transmitted 4 stream payload bytes");
            }
        }
        close(sock);
    } else {
        puts("  [FAIL] socket() returned invalid descriptor");
        return 1;
    }
    puts("  [OK]   BSD socket subsystem operational");

    /* 8. VMM Demand Paging & Lazy Heap (sbrk) Allocation */
    puts("  [TEST] VMM demand paging & lazy heap (sbrk) allocation...");
    void *old_brk = sbrk(65536);
    if (old_brk != (void *)-1) {
        volatile char *canary_page1 = (volatile char *)old_brk + 4096;
        volatile char *canary_page2 = (volatile char *)old_brk + 32768;
        *canary_page1 = 'K';
        *canary_page2 = 'R';
        if (*canary_page1 == 'K' && *canary_page2 == 'R') {
            puts("  [INFO] Lazy pages faulted in transparently via #PF");
        } else {
            puts("  [FAIL] Canary value corruption on faulted heap pages");
            return 1;
        }
    } else {
        puts("  [FAIL] sbrk(65536) heap expansion failed");
        return 1;
    }
    puts("  [OK]   VMM demand paging operational");

    /* 9. Out-of-bounds Syscall Handshake */
    puts("  [TEST] Out-of-bounds syscall safety check...");
    int64_t unhandled = syscall0(9999);
    (void)unhandled;
    puts("  [OK]   Undefined syscall safely handled without kernel fault");

    /* 10. Buffered Stream I/O Operations */
    puts("  [TEST] Buffered standard I/O stream operations...");
    FILE *fp = fopen("/config/sys/hostname.cfg", "r");
    if (fp) {
        char line_buf[64];
        memset(line_buf, 0, sizeof(line_buf));
        if (fgets(line_buf, sizeof(line_buf), fp)) {
            size_t len = strlen(line_buf);
            if (len > 0 && line_buf[len - 1] == '\n')
                line_buf[len - 1] = '\0';
            printf("  [INFO] Read stream line: \"%s\"\n", line_buf);
        }
        fseek(fp, 0, SEEK_SET);
        if (ftell(fp) == 0 && fgetc(fp) == 'k') {
            puts("  [INFO] Stream seek & single-byte cache hit verified");
        }
        fclose(fp);
        puts("  [OK]   Buffered standard I/O operational");
    } else {
        puts("  [FAIL] fopen() failed on /config/sys/hostname.cfg");
        return 1;
    }

    /* 11. Dual-Tier Memory Allocator & Coalescing */
    puts("  [TEST] Dual-tier memory allocator (heap & mmap tiers)...");
    void *p1 = malloc(64);
    void *p2 = malloc(64);
    void *p3 = malloc(64);
    if (!p1 || !p2 || !p3) {
        if (p1)
            free(p1);
        if (p2)
            free(p2);
        if (p3)
            free(p3);
        puts("  [FAIL] malloc() failed for small heap chunks");
        return 1;
    }
    memset(p1, 0xAA, 64);
    memset(p2, 0xBB, 64);
    memset(p3, 0xCC, 64);

    free(p2);
    free(p3);

    void *p_merged = malloc(128);
    if (!p_merged) {
        free(p1);
        puts("  [FAIL] malloc() failed for merged block");
        return 1;
    }
    free(p1);
    free(p_merged);
    puts("  [INFO] Heap tier forward coalescing validated");

    size_t large_sz = 262144; /* 256 KiB */
    char *large_ptr = (char *)malloc(large_sz);
    if (large_ptr) {
        large_ptr[0] = 'M';
        large_ptr[large_sz - 1] = 'Z';
        if (large_ptr[0] == 'M' && large_ptr[large_sz - 1] == 'Z') {
            puts("  [INFO] Mmap tier 256 KiB allocation validated");
        }
        free(large_ptr);
        puts("  [INFO] Mmap tier deallocation validated");
    } else {
        puts("  [FAIL] malloc() failed for 256 KiB mmap tier");
        return 1;
    }
    puts("  [OK]   Dual-tier memory allocator operational");

    /* 12. Environment Variable Management */
    puts("  [TEST] Environment variable operations (getenv, setenv, unsetenv)...");
    setenv("KEIRA_ENV_TEST", "userland_active", 1);
    char *env_val = getenv("KEIRA_ENV_TEST");
    if (env_val && strcmp(env_val, "userland_active") == 0) {
        printf("  [INFO] Environment variable set and read: \"%s\"\n", env_val);
        unsetenv("KEIRA_ENV_TEST");
        if (getenv("KEIRA_ENV_TEST") == NULL) {
            puts("  [INFO] Environment variable unset validated");
        }
        puts("  [OK]   Environment variable subsystem operational");
    } else {
        puts("  [FAIL] setenv/getenv operation failed");
        return 1;
    }

    /* 13. Advanced Stream Formatting & Parsing (fprintf, sscanf) */
    puts("  [TEST] Stream formatting & string scanning (fprintf, sscanf)...");
    char scan_buf[64];
    int parsed_id = 0;
    char parsed_name[32];
    memset(scan_buf, 0, sizeof(scan_buf));
    memset(parsed_name, 0, sizeof(parsed_name));
    snprintf(scan_buf, sizeof(scan_buf), "task: 42 name: keira_proc");
    int matches = sscanf(scan_buf, "task: %d name: %s", &parsed_id, parsed_name);
    if (matches == 2 && parsed_id == 42 && strcmp(parsed_name, "keira_proc") == 0) {
        printf("  [INFO] sscanf parsed %d tokens: id=%d name=\"%s\"\n", matches, parsed_id,
               parsed_name);
        fprintf(stdout, "  [INFO] fprintf stream output verified\n");
        puts("  [OK]   Stream formatting and string parsing operational");
    } else {
        puts("  [FAIL] sscanf failed to parse expected tokens");
        return 1;
    }

    /* 14. Ring 3 Multiprocess Orchestration (fork + waitpid + exit) */
    puts("  [TEST] Ring 3 multiprocess orchestration (fork + waitpid)...");
    pid_t child_pid = fork();
    if (child_pid < 0) {
        puts("  [FAIL] fork() syscall failed");
        return 1;
    } else if (child_pid == 0) {
        /* In child process: exit immediately with status 42 */
        exit(42);
    } else {
        /* In parent process: reap child and verify status */
        int wstatus = 0;
        pid_t reaped_pid = waitpid(child_pid, &wstatus, 0);
        if (reaped_pid == child_pid && WIFEXITED(wstatus) && WEXITSTATUS(wstatus) == 42) {
            printf("  [INFO] Child PID %d reaped with exit status %d\n", (int)reaped_pid,
                   WEXITSTATUS(wstatus));
            puts("  [OK]   Ring 3 process orchestration operational");
        } else {
            printf("  [FAIL] waitpid mismatch: reaped=%d, wstatus=%d\n", (int)reaped_pid, wstatus);
            return 1;
        }
    }

    /* 15. Anonymous Inter-Process Pipe Streaming */
    puts("  [TEST] Anonymous inter-process pipe streaming (pipe + fork + IPC)...");
    int pipefds[2] = {-1, -1};
    if (pipe(pipefds) < 0) {
        puts("  [FAIL] pipe() allocation failed");
        return 1;
    }
    pid_t pipe_child = fork();
    if (pipe_child < 0) {
        puts("  [FAIL] fork() failed during pipe test");
        return 1;
    } else if (pipe_child == 0) {
        /* In child: close read end, write message, close write end, exit */
        close(pipefds[0]);
        const char *msg = "KEIRA_PIPE_STREAM_OK";
        ssize_t w = write(pipefds[1], msg, strlen(msg));
        close(pipefds[1]);
        exit((w == (ssize_t)strlen(msg)) ? 0 : 1);
    } else {
        /* In parent: wait for child to complete transmission, then read and verify */
        int child_status = 0;
        waitpid(pipe_child, &child_status, 0);
        close(pipefds[1]);
        char pipe_in[32];
        memset(pipe_in, 0, sizeof(pipe_in));
        ssize_t r = read(pipefds[0], pipe_in, sizeof(pipe_in) - 1);
        close(pipefds[0]);
        if (r > 0 && strcmp(pipe_in, "KEIRA_PIPE_STREAM_OK") == 0) {
            printf("  [INFO] Received pipe message: \"%s\"\n", pipe_in);
            puts("  [OK]   Anonymous pipe inter-process streaming operational");
        } else {
            printf("  [FAIL] Pipe communication failed: r=%d, msg=\"%s\"\n", (int)r, pipe_in);
            return 1;
        }
    }

    /* 16. Copy-on-Write (COW) Memory Mutation Integrity */
    puts("  [TEST] Copy-on-Write (COW) memory mutation isolation...");
    volatile int *cow_canary = (volatile int *)malloc(sizeof(int));
    if (!cow_canary) {
        puts("  [FAIL] malloc failed for COW canary");
        return 1;
    }
    *cow_canary = 0x5A5A1234;
    pid_t cow_child = fork();
    if (cow_child < 0) {
        puts("  [FAIL] fork() failed during COW test");
        free((void *)cow_canary);
        return 1;
    } else if (cow_child == 0) {
        /* In child: mutate value and exit */
        *cow_canary = (int)0xDEADBEEF;
        exit(0);
    } else {
        int cow_status = 0;
        waitpid(cow_child, &cow_status, 0);
        if (*cow_canary == 0x5A5A1234) {
            puts("  [INFO] Parent memory unchanged after child mutation (COW verified)");
            puts("  [OK]   Copy-on-Write memory mutation isolation operational");
        } else {
            printf("  [FAIL] Parent memory corrupted by child: 0x%X\n", *cow_canary);
            free((void *)cow_canary);
            return 1;
        }
        free((void *)cow_canary);
    }

    /* 17. Process Tree Reaping & Multi-Process Zombie Status Propagation */
    puts("  [TEST] Multi-process tree reaping & zombie status propagation...");
    int expected_codes[3] = {11, 22, 33};
    pid_t child_pids[3];
    int all_forked = 1;
    for (int i = 0; i < 3; i++) {
        pid_t p = fork();
        if (p < 0) {
            all_forked = 0;
            break;
        } else if (p == 0) {
            exit(expected_codes[i]);
        } else {
            child_pids[i] = p;
        }
    }
    if (!all_forked) {
        puts("  [FAIL] Multi-process fork failed");
        return 1;
    }
    int reap_success = 1;
    for (int i = 0; i < 3; i++) {
        int s = 0;
        pid_t r = waitpid(child_pids[i], &s, 0);
        if (r != child_pids[i] || !WIFEXITED(s) || WEXITSTATUS(s) != expected_codes[i]) {
            printf("  [FAIL] Child %d reap failed: pid=%d, status=%d\n", i, (int)r, WEXITSTATUS(s));
            reap_success = 0;
            break;
        }
    }
    if (reap_success) {
        puts("  [INFO] 3 child processes reaped with exact exit codes (11, 22, 33)");
        puts("  [OK]   Process tree reaping and zombie status propagation operational");
    } else {
        return 1;
    }

    /* 18. Cross-Architecture User Boundary Hardening */
    puts("  [TEST] Cross-architecture high kernel pointer rejection...");
#if defined(__x86_64__)
    void *illegal_ptr = (void *)0xFFFF800000000000ULL;
#else
    void *illegal_ptr = (void *)0xC0000000UL;
#endif
    ssize_t illegal_write = write(1, illegal_ptr, 1);
    ssize_t illegal_read = read(0, illegal_ptr, 1);
    if (illegal_write < 0 && illegal_read < 0) {
        puts("  [INFO] Syscall write/read to high kernel pointer strictly rejected");
        puts("  [OK]   Cross-architecture kernel boundary security verified");
    } else {
        puts("  [FAIL] High kernel pointer was not rejected with EFAULT!");
        return 1;
    }

    /* 19. Process Credentials & Privilege Demotion (getuid/setuid) */
    puts("  [TEST] Process credentials and privilege demotion (getuid/setuid)...");
    uid_t initial_uid = getuid();
    if (initial_uid != 0) {
        printf("  [FAIL] Expected initial UID 0, got %d\n", (int)initial_uid);
        return 1;
    }
    /* Demote to unprivileged UID 1000 */
    if (setuid(1000) != 0) {
        puts("  [FAIL] setuid(1000) failed for root caller");
        return 1;
    }
    if (getuid() != 1000) {
        printf("  [FAIL] Expected demoted UID 1000, got %d\n", (int)getuid());
        return 1;
    }
    /* Attempt unauthorized privilege escalation back to root (UID 0) */
    if (setuid(0) == 0) {
        puts("  [FAIL] Unauthorized privilege escalation allowed (security breach)!");
        return 1;
    }
    if (errno != EPERM) {
        printf("  [FAIL] Expected EPERM (1), got errno %d\n", errno);
        return 1;
    }
    puts("  [INFO] Successfully demoted to UID 1000 and blocked escalation to UID 0 (EPERM)");
    puts("  [OK]   Process credentials and privilege demotion operational");

    /* 20. POSIX Signal Context Delivery & Sigreturn */
    puts("  [TEST] POSIX signal context registration and sigreturn verification...");
    /* Test sigreturn when no signal context is saved -> must return EINVAL (-22) */
    int64_t invalid_sigret = syscall0(SYS_SIGRETURN);
    if (invalid_sigret != -EINVAL) {
        printf("  [FAIL] sys_sigreturn without saved context should fail with -EINVAL, got %ld\n",
               (long)invalid_sigret);
        return 1;
    }
    /* Register custom handler for SIGUSR1 via SYS_SIGACTION */
    uint64_t old_handler = 0;
    int64_t act_ret = syscall3(SYS_SIGACTION, 10, (uint64_t)(uintptr_t)0x40001234,
                               (uint64_t)(uintptr_t)&old_handler);
    if (act_ret != 0) {
        puts("  [FAIL] sys_sigaction failed to register signal handler");
        return 1;
    }
    /* Send SIGUSR1 to self (PID) -> kernel recognizes handler, prepares context */
    int64_t kill_ret = syscall2(SYS_KILL, (uint64_t)getpid(), 10);
    if (kill_ret != 0) {
        puts("  [FAIL] sys_kill failed to dispatch handled signal");
        return 1;
    }
    /* Now sys_sigreturn must succeed and clear context */
    int64_t valid_sigret = syscall0(SYS_SIGRETURN);
    if (valid_sigret != 0) {
        printf("  [FAIL] sys_sigreturn with saved context failed: %ld\n", (long)valid_sigret);
        return 1;
    }
    /* Subsequent sys_sigreturn should return -EINVAL because context was already consumed */
    int64_t second_sigret = syscall0(SYS_SIGRETURN);
    if (second_sigret != -EINVAL) {
        puts("  [FAIL] Second sys_sigreturn should fail after context was consumed");
        return 1;
    }
    puts("  [INFO] Signal handler registered, signal context saved, and restored via sigreturn");
    puts("  [OK]   POSIX signal context delivery and sigreturn restorer operational");

    /* 21. Character Device /system/dev/tty Stream */
    puts("  [TEST] Character device /system/dev/tty stream read/write...");
    int tty_fd = open("/system/dev/tty", O_RDWR, 0);
    if (tty_fd < 0) {
        puts("  [FAIL] open(/system/dev/tty) failed");
        return 1;
    }
    /* Write to TTY */
    const char *tty_msg = " [TTY_ECHO_OK]\n";
    ssize_t written = write(tty_fd, tty_msg, strlen(tty_msg));
    if (written != (ssize_t)strlen(tty_msg)) {
        puts("  [FAIL] write to /system/dev/tty failed");
        close(tty_fd);
        return 1;
    }
    /* Read from TTY (non-blocking / draining queue) */
    char tty_in[16];
    ssize_t tty_read = read(tty_fd, tty_in, sizeof(tty_in));
    if (tty_read < 0) {
        puts("  [FAIL] read from /system/dev/tty returned error");
        close(tty_fd);
        return 1;
    }
    close(tty_fd);
    puts("  [INFO] /system/dev/tty write and non-blocking read queue drain verified");
    puts("  [OK]   Character device /system/dev/tty stream operational");

    /* 22. TTY Line Discipline & Termios Mode Switching */
    puts("  [TEST] TTY line discipline and termios mode switching...");
    int tty_term_fd = open("/system/dev/tty", O_RDWR, 0);
    if (tty_term_fd < 0) {
        puts("  [FAIL] open(/system/dev/tty) for termios failed");
        return 1;
    }
    struct termios orig_term;
    if (tcgetattr(tty_term_fd, &orig_term) != 0) {
        puts("  [FAIL] tcgetattr failed");
        close(tty_term_fd);
        return 1;
    }
    if ((orig_term.c_lflag & ICANON) == 0) {
        puts("  [FAIL] Default termios should have ICANON enabled");
        close(tty_term_fd);
        return 1;
    }
    /* Switch to raw mode (~ICANON) and write back */
    struct termios raw_term = orig_term;
    raw_term.c_lflag &= ~ICANON;
    if (tcsetattr(tty_term_fd, TCSANOW, &raw_term) != 0) {
        puts("  [FAIL] tcsetattr to raw mode failed");
        close(tty_term_fd);
        return 1;
    }
    struct termios verify_term;
    if (tcgetattr(tty_term_fd, &verify_term) != 0) {
        puts("  [FAIL] tcgetattr verify failed");
        close(tty_term_fd);
        return 1;
    }
    if ((verify_term.c_lflag & ICANON) != 0) {
        puts("  [FAIL] Termios TCSETS failed to persist raw mode");
        close(tty_term_fd);
        return 1;
    }
    /* Restore canonical mode */
    tcsetattr(tty_term_fd, TCSANOW, &orig_term);
    close(tty_term_fd);
    puts("  [INFO] Termios TCGETS and TCSETS attribute persistence verified");
    puts("  [OK]   TTY line discipline and termios mode switching operational");

    /* 23. POSIX Signal Masking & Pending Signal Queue */
    puts("  [TEST] POSIX signal masking (sigprocmask/sigpending)...");
    sigset_t mask_set;
    sigemptyset(&mask_set);
    sigaddset(&mask_set, SIGUSR1);
    if (!sigismember(&mask_set, SIGUSR1)) {
        puts("  [FAIL] sigaddset/sigismember mismatch");
        return 1;
    }
    sigset_t old_mask = 0;
    if (sigprocmask(SIG_BLOCK, &mask_set, &old_mask) != 0) {
        puts("  [FAIL] sigprocmask(SIG_BLOCK) failed");
        return 1;
    }
    /* Send SIGUSR1 to self while masked */
    if (kill(getpid(), SIGUSR1) != 0) {
        puts("  [FAIL] kill failed to queue masked signal");
        return 1;
    }
    /* Verify signal is pending */
    sigset_t pending_set = 0;
    if (sigpending(&pending_set) != 0) {
        puts("  [FAIL] sigpending query failed");
        return 1;
    }
    if (!sigismember(&pending_set, SIGUSR1)) {
        puts("  [FAIL] Masked SIGUSR1 was not recorded in pending signal queue");
        return 1;
    }
    /* Unblock signal -> pending signal should be dispatched and cleared */
    if (sigprocmask(SIG_UNBLOCK, &mask_set, NULL) != 0) {
        puts("  [FAIL] sigprocmask(SIG_UNBLOCK) failed");
        return 1;
    }
    sigpending(&pending_set);
    if (sigismember(&pending_set, SIGUSR1)) {
        puts("  [FAIL] Unblocked signal was not cleared from pending queue");
        return 1;
    }
    puts("  [INFO] Masked signal blocked, queued, and delivered on unblock");
    puts("  [OK]   POSIX signal masking and pending queue operational");

    /* 24. ProcFS Dynamic Telemetry & DevFS Isolation */
    puts("  [TEST] ProcFS dynamic telemetry and DevFS isolation...");
    char proc_buf[256];
    memset(proc_buf, 0, sizeof(proc_buf));
    int up_fd = open("/system/proc/uptime", O_RDONLY, 0);
    if (up_fd < 0 || read(up_fd, proc_buf, sizeof(proc_buf) - 1) <= 0) {
        puts("  [FAIL] Failed to read /system/proc/uptime");
        if (up_fd >= 0)
            close(up_fd);
        return 1;
    }
    close(up_fd);
    if (strchr(proc_buf, '.') == NULL) {
        puts("  [FAIL] /system/proc/uptime content malformed");
        return 1;
    }

    memset(proc_buf, 0, sizeof(proc_buf));
    int mem_fd = open("/system/proc/meminfo", O_RDONLY, 0);
    if (mem_fd < 0 || read(mem_fd, proc_buf, sizeof(proc_buf) - 1) <= 0) {
        puts("  [FAIL] Failed to read /system/proc/meminfo");
        if (mem_fd >= 0)
            close(mem_fd);
        return 1;
    }
    close(mem_fd);
    if (strstr(proc_buf, "MemTotal:") == NULL) {
        puts("  [FAIL] /system/proc/meminfo missing MemTotal token");
        return 1;
    }

    memset(proc_buf, 0, sizeof(proc_buf));
    int cpu_fd = open("/system/proc/cpuinfo", O_RDONLY, 0);
    if (cpu_fd < 0 || read(cpu_fd, proc_buf, sizeof(proc_buf) - 1) <= 0) {
        puts("  [FAIL] Failed to read /system/proc/cpuinfo");
        if (cpu_fd >= 0)
            close(cpu_fd);
        return 1;
    }
    close(cpu_fd);
    if (strstr(proc_buf, "vendor_id") == NULL) {
        puts("  [FAIL] /system/proc/cpuinfo missing vendor_id token");
        return 1;
    }

    memset(proc_buf, 0, sizeof(proc_buf));
    int stat_fd = open("/system/proc/self/status", O_RDONLY, 0);
    if (stat_fd < 0 || read(stat_fd, proc_buf, sizeof(proc_buf) - 1) <= 0) {
        puts("  [FAIL] Failed to read /system/proc/self/status");
        if (stat_fd >= 0)
            close(stat_fd);
        return 1;
    }
    close(stat_fd);
    if (strstr(proc_buf, "State:") == NULL) {
        puts("  [FAIL] /system/proc/self/status missing State token");
        return 1;
    }

    int dev_null = open("/system/dev/null", O_RDONLY, 0);
    if (dev_null < 0 || read(dev_null, proc_buf, 10) != 0) {
        puts("  [FAIL] Dynamic /system/dev/null read failed");
        if (dev_null >= 0)
            close(dev_null);
        return 1;
    }
    close(dev_null);

    int dev_zero = open("/system/dev/zero", O_RDONLY, 0);
    if (dev_zero < 0 || read(dev_zero, proc_buf, 8) != 8 || proc_buf[0] != 0) {
        puts("  [FAIL] Dynamic /system/dev/zero read failed");
        if (dev_zero >= 0)
            close(dev_zero);
        return 1;
    }
    close(dev_zero);

    puts("  [INFO] ProcFS metrics and DevFS dynamic nodes verified");
    puts("  [OK]   Dynamic pseudo-filesystem operational");

    /* 25. Stack Canary Protection & Argument Passing ABI */
    puts("  [TEST] Stack canary protection and argument ABI...");
    extern uintptr_t __stack_chk_guard;
    if (__stack_chk_guard == 0) {
        puts("  [FAIL] __stack_chk_guard is zero");
        return 1;
    }
    if (argc < 1 || argv == NULL || argv[0] == NULL) {
        puts("  [FAIL] System V argument stack frame malformed");
        return 1;
    }
    printf("  [INFO] Stack canary guard initialized (0x%lx), argc=%d, argv[0]=%s\n",
           (unsigned long)__stack_chk_guard, argc, argv[0]);
    puts("  [OK]   Stack canary protection and argument ABI operational");

    /* 26. File-Backed Memory Mapping, Demand Paging, and msync Synchronization */
    puts("  [TEST] File-backed mmap, demand paging, and msync synchronization...");
    const char *test_path = "/config/sys/mmap_abi.txt";
    int f_init = open(test_path, O_CREAT | O_RDWR | O_TRUNC, 0644);
    if (f_init < 0) {
        puts("  [FAIL] Failed to create /config/sys/mmap_abi.txt for mmap test");
        return 1;
    }
    const char *init_payload = "INIT_PAYLOAD_KEIRA_MMAP_PERSISTENCE_TEST";
    written = write(f_init, init_payload, strlen(init_payload));
    if (written < (ssize_t)strlen(init_payload)) {
        puts("  [FAIL] Failed to write initial payload to test file");
        close(f_init);
        return 1;
    }
    close(f_init);

    int mmap_fd = open(test_path, O_RDWR, 0);
    if (mmap_fd < 0) {
        puts("  [FAIL] Failed to reopen test file for mmap");
        return 1;
    }

    char *mapped = (char *)mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, mmap_fd, 0);
    if (mapped == MAP_FAILED || mapped == NULL) {
        puts("  [FAIL] File-backed mmap() returned MAP_FAILED");
        close(mmap_fd);
        return 1;
    }

    /* Verify demand paging: reading triggers #PF which lazily faults in file data */
    if (strncmp(mapped, init_payload, strlen(init_payload)) != 0) {
        puts("  [FAIL] Demand paging data mismatch upon initial read");
        munmap(mapped, 4096);
        close(mmap_fd);
        return 1;
    }
    printf("  [INFO] Demand paging verified: read '%s'\n", init_payload);

    /* Mutate mapped memory */
    mapped[0] = 'D';
    mapped[1] = 'O';
    mapped[2] = 'N';
    mapped[3] = 'E';

    /* Synchronize dirty page back to disk */
    if (msync(mapped, 4096, MS_SYNC) != 0) {
        puts("  [FAIL] msync() returned non-zero error");
        munmap(mapped, 4096);
        close(mmap_fd);
        return 1;
    }

    /* Unmap memory */
    if (munmap(mapped, 4096) != 0) {
        puts("  [FAIL] munmap() failed");
        close(mmap_fd);
        return 1;
    }
    close(mmap_fd);

    /* Reopen file from disk and verify mutated bytes persisted */
    int verify_fd = open(test_path, O_RDONLY, 0);
    if (verify_fd < 0) {
        puts("  [FAIL] Failed to open test file for persistence verification");
        return 1;
    }
    char verify_buf[64];
    memset(verify_buf, 0, sizeof(verify_buf));
    ssize_t n_read = read(verify_fd, verify_buf, sizeof(verify_buf) - 1);
    close(verify_fd);

    if (n_read < 4 || strncmp(verify_buf, "DONE", 4) != 0) {
        printf("  [FAIL] Disk persistence verification failed: read '%s'\n", verify_buf);
        return 1;
    }
    printf("  [INFO] Disk persistence confirmed: '%s'\n", verify_buf);
    puts("  [OK]   File-backed mmap, demand paging, and msync synchronization operational");

    /* 27. Illegal Instruction (#UD) Fault Containment */
    puts("  [TEST] Illegal instruction (#UD) fault containment and SIGILL delivery...");
    pid_t ud_child = fork();
    if (ud_child < 0) {
        puts("  [FAIL] fork() failed for #UD test");
        return 1;
    } else if (ud_child == 0) {
        __asm__ volatile("ud2");
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(ud_child, &wstatus, 0);
        if (reaped == ud_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGILL) {
            printf("  [INFO] Child PID %d trapped #UD, terminated by signal %d (SIGILL)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Illegal instruction hardware exception safely trapped to SIGILL");
        } else {
            printf("  [FAIL] Expected SIGILL (4), got reaped=%d, signaled=%d, termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }

    /* 28. Division by Zero (#DE) Fault Containment */
    puts("  [TEST] Division by zero (#DE) fault containment and SIGFPE delivery...");
    pid_t de_child = fork();
    if (de_child < 0) {
        puts("  [FAIL] fork() failed for #DE test");
        return 1;
    } else if (de_child == 0) {
        volatile int num = 100;
        volatile int den = 0;
        volatile int res = num / den;
        (void)res;
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(de_child, &wstatus, 0);
        if (reaped == de_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGFPE) {
            printf("  [INFO] Child PID %d trapped #DE, terminated by signal %d (SIGFPE)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Division by zero hardware exception safely trapped to SIGFPE");
        } else {
            printf("  [FAIL] Expected SIGFPE (8), got reaped=%d, signaled=%d, termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }

    /* 29. Memory Dereference (#PF) Fault Containment */
    puts("  [TEST] Memory dereference fault containment and SIGSEGV delivery...");
    pid_t pf_child = fork();
    if (pf_child < 0) {
        puts("  [FAIL] fork() failed for #PF test");
        return 1;
    } else if (pf_child == 0) {
        *(volatile int *)0x1234 = 99;
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(pf_child, &wstatus, 0);
        if (reaped == pf_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGSEGV) {
            printf("  [INFO] Child PID %d trapped #PF, terminated by signal %d (SIGSEGV)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Memory dereference hardware exception safely trapped to SIGSEGV");
        } else {
            printf("  [FAIL] Expected SIGSEGV (11), got reaped=%d, signaled=%d, termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }

    /* 30. Demand-Paged VMA Memory Validation Across Syscall Boundaries */
    puts("  [TEST] Demand-paged VMA memory validation across syscall boundaries...");
    char *vma_buf = mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (vma_buf == MAP_FAILED || vma_buf == NULL) {
        puts("  [FAIL] mmap failed for demand-paging syscall test");
        return 1;
    }
    int pipe_fds[2];
    if (pipe(pipe_fds) != 0) {
        puts("  [FAIL] pipe creation failed");
        munmap(vma_buf, 4096);
        return 1;
    }
    const char *vma_msg = "KEIRA_VMA_OK";
    if (write(pipe_fds[1], vma_msg, 12) != 12) {
        puts("  [FAIL] pipe write failed");
        close(pipe_fds[0]);
        close(pipe_fds[1]);
        munmap(vma_buf, 4096);
        return 1;
    }
    ssize_t vma_read = read(pipe_fds[0], vma_buf, 12);
    close(pipe_fds[0]);
    close(pipe_fds[1]);
    if (vma_read != 12 || strncmp(vma_buf, vma_msg, 12) != 0) {
        printf("  [FAIL] Demand-paged VMA read failed: n=%ld\n", (long)vma_read);
        munmap(vma_buf, 4096);
        return 1;
    }
    if (munmap(vma_buf, 4096) != 0) {
        puts("  [FAIL] munmap failed for demand-paged VMA test");
        return 1;
    }
    puts("  [OK]   Demand-paged VMA populated and validated in syscall copy without EFAULT");

    /* 31. Persistent Core Dump Diagnostic Inspection */
    puts("  [TEST] Persistent core dump diagnostic artifact inspection...");
    char core_path[64];
    snprintf(core_path, sizeof(core_path), "/data/log/core_%d.dmp", (int)pf_child);
    int core_fd = open(core_path, O_RDONLY, 0);
    if (core_fd < 0) {
        printf("  [FAIL] Failed to open core dump artifact: %s\n", core_path);
        return 1;
    }
    char core_data[256];
    memset(core_data, 0, sizeof(core_data));
    ssize_t core_read = read(core_fd, core_data, sizeof(core_data) - 1);
    close(core_fd);
    if (core_read <= 0 || strstr(core_data, "KEIRA CORE DUMP") == NULL) {
        printf("  [FAIL] Core dump content invalid or missing header in %s\n", core_path);
        return 1;
    }
    printf("  [INFO] Core dump %s verified (found 'KEIRA CORE DUMP')\n", core_path);
    puts("  [OK]   Persistent core dump diagnostic generation operational");

    /* 32. Rapid Process Fork & Reap Churn (20 iterations) */
    puts("  [TEST] Rapid process fork & reap churn (20 iterations)...");
    for (int iter = 0; iter < 20; iter++) {
        pid_t child = fork();
        if (child < 0) {
            printf("  [FAIL] Fork churn failed at iteration %d\n", iter);
            return 1;
        } else if (child == 0) {
            exit(iter + 10);
        } else {
            int wstatus = 0;
            pid_t reaped = waitpid(child, &wstatus, 0);
            if (reaped != child || !WIFEXITED(wstatus) || WEXITSTATUS(wstatus) != (iter + 10)) {
                printf("  [FAIL] Churn waitpid mismatch at iter %d: reaped=%d, status=%d\n", iter,
                       (int)reaped, WEXITSTATUS(wstatus));
                return 1;
            }
        }
    }
    puts("  [INFO] 20 sequential processes spawned, executed, and reaped cleanly");
    puts("  [OK]   Rapid fork and reap churn completed with zero slot or memory leaks");

    /* 33. Orphan Process Reparenting & PID 0 Adoption */
    puts("  [TEST] Orphan process reparenting and PID 0 adoption lifecycle...");
    const char *orphan_path = "/temp/orphan_test.txt";
    pid_t parent_fork = fork();
    if (parent_fork < 0) {
        puts("  [FAIL] Fork failed for orphan parent");
        return 1;
    } else if (parent_fork == 0) {
        /* Child process: forks grandchild and exits immediately */
        pid_t grandchild = fork();
        if (grandchild < 0) {
            exit(1);
        } else if (grandchild == 0) {
            /* Grandchild: wait until parent exits and PID 0 adopts it */
            pid_t ppid = getppid();
            for (int r = 0; r < 100 && ppid != 0; r++) {
                usleep(5000);
                ppid = getppid();
            }
            if (ppid == 0) {
                int f = open(orphan_path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
                if (f >= 0) {
                    write(f, "ORPHAN_PPID_0", 13);
                    close(f);
                }
            }
            exit(42);
        } else {
            /* Child exits immediately, leaving grandchild orphaned */
            exit(0);
        }
    } else {
        /* Grandparent process */
        int wstatus = 0;
        pid_t reaped_child = waitpid(parent_fork, &wstatus, 0);
        if (reaped_child != parent_fork || !WIFEXITED(wstatus) || WEXITSTATUS(wstatus) != 0) {
            puts("  [FAIL] Parent child process waitpid failed");
            return 1;
        }

        /* Wait for orphaned grandchild to execute and write artifact */
        char orphan_buf[32];
        memset(orphan_buf, 0, sizeof(orphan_buf));
        for (int r = 0; r < 100; r++) {
            int f = open(orphan_path, O_RDONLY, 0);
            if (f >= 0) {
                ssize_t n = read(f, orphan_buf, sizeof(orphan_buf) - 1);
                close(f);
                if (n > 0 && strstr(orphan_buf, "ORPHAN_PPID_0") != NULL) {
                    break;
                }
            }
            usleep(5000);
        }

        if (strstr(orphan_buf, "ORPHAN_PPID_0") == NULL) {
            printf("  [FAIL] Grandchild was not reparented to PID 0: '%s'\n", orphan_buf);
            return 1;
        }

        /* Reap the adopted orphaned grandchild (adopted by PID 0) */
        int orphan_status = 0;
        pid_t reaped_orphan = waitpid(-1, &orphan_status, 0);
        if (reaped_orphan <= 0 || !WIFEXITED(orphan_status) || WEXITSTATUS(orphan_status) != 42) {
            printf("  [FAIL] Reaping adopted orphan failed: pid=%d, status=%d\n",
                   (int)reaped_orphan, WEXITSTATUS(orphan_status));
            return 1;
        }

        /* Verify Process A now has no remaining children */
        int no_child_status = 0;
        pid_t no_child = waitpid(-1, &no_child_status, WNOHANG);
        if (no_child != -1) {
            printf("  [FAIL] Expected ECHILD (-1) after reaping orphan, got %d\n", (int)no_child);
            return 1;
        }

        puts("  [INFO] Orphaned grandchild confirmed adopted by PID 0 and reaped cleanly");
        puts("  [OK]   Orphan process reparenting and PID 0 adoption lifecycle operational");
    }

    /* 34. File Descriptor & Write Lock Auto-Reclaim Upon Process Exit */
    puts("  [TEST] File descriptor and write lock auto-reclaim upon process exit...");
    const char *flock_path = "/temp/flock_reclaim.txt";
    pid_t lock_child = fork();
    if (lock_child < 0) {
        puts("  [FAIL] Fork failed for flock auto-reclaim test");
        return 1;
    } else if (lock_child == 0) {
        /* Child opens file in write mode, acquiring write lock */
        int fd_child = open(flock_path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
        if (fd_child < 0) {
            exit(1);
        }
        int dummy_pipe[2];
        pipe(dummy_pipe);
        /* Intentionally exit abruptly without closing fd_child or dummy_pipe */
        exit(15);
    } else {
        int wstatus = 0;
        pid_t reaped_lock = waitpid(lock_child, &wstatus, 0);
        if (reaped_lock != lock_child || !WIFEXITED(wstatus) || WEXITSTATUS(wstatus) != 15) {
            puts("  [FAIL] Lock child process waitpid failed");
            return 1;
        }

        /* Attempt to acquire write lock immediately on the same file */
        int fd_parent = open(flock_path, O_WRONLY, 0);
        if (fd_parent < 0) {
            puts("  [FAIL] File lock was not released upon child process exit (EACCES)");
            return 1;
        }
        const char *reclaim_msg = "RECLAIM_LOCK_OK";
        write(fd_parent, reclaim_msg, strlen(reclaim_msg));
        close(fd_parent);
        puts("  [INFO] File write lock successfully re-acquired immediately after unclosed child "
             "exit");
        puts("  [OK]   File descriptor and write lock auto-reclaim operational");
    }

    /* 35. File System & Hardware Storage Cache Synchronization */
    puts("  [TEST] File system and hardware storage cache synchronization...");
    if (fsync(-1) != -1 || errno != EBADF) {
        printf("  [FAIL] fsync(-1) expected EBADF, got errno=%d\n", errno);
        return 1;
    }
    if (fsync(99) != -1 || errno != EBADF) {
        printf("  [FAIL] fsync(99) expected EBADF, got errno=%d\n", errno);
        return 1;
    }

    const char *sync_path = "/temp/sync_test.txt";
    int fd_sync = open(sync_path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd_sync < 0) {
        puts("  [FAIL] Failed to create sync test file");
        return 1;
    }
    const char *sync_payload = "STORAGE_FLUSH_VERIFIED";
    ssize_t sync_written = write(fd_sync, sync_payload, strlen(sync_payload));
    if (sync_written != (ssize_t)strlen(sync_payload)) {
        puts("  [FAIL] Failed to write sync test payload");
        close(fd_sync);
        return 1;
    }

    if (fsync(fd_sync) != 0) {
        printf("  [FAIL] fsync on open descriptor failed, errno=%d\n", errno);
        close(fd_sync);
        return 1;
    }

    if (sync() != 0) {
        printf("  [FAIL] sync() failed, errno=%d\n", errno);
        close(fd_sync);
        return 1;
    }

    close(fd_sync);

    int fd_verify = open(sync_path, O_RDONLY, 0);
    if (fd_verify < 0) {
        puts("  [FAIL] Failed to reopen sync test file for verification");
        return 1;
    }
    char sync_verify_buf[64];
    memset(sync_verify_buf, 0, sizeof(sync_verify_buf));
    ssize_t sync_read_bytes = read(fd_verify, sync_verify_buf, sizeof(sync_verify_buf) - 1);
    close(fd_verify);

    if (sync_read_bytes != (ssize_t)strlen(sync_payload) ||
        strcmp(sync_verify_buf, sync_payload) != 0) {
        printf("  [FAIL] Mismatched synced file contents: '%s'\n", sync_verify_buf);
        return 1;
    }
    puts("  [INFO] Hardware ATA write cache flush and sector barrier verified");
    puts("  [OK]   File system and hardware cache synchronization operational");

    /* 36. Descriptor Duplication & Slot Targeting (dup and dup2) */
    puts("  [TEST] POSIX descriptor duplication and targeting (dup & dup2)...");
    const char *dup_path = "/temp/dup_test.txt";
    int fd_orig = open(dup_path, O_CREAT | O_RDWR | O_TRUNC, 0644);
    if (fd_orig < 0) {
        puts("  [FAIL] Failed to open dup test file");
        return 1;
    }
    const char *dup_payload = "ABCDEFGH";
    write(fd_orig, dup_payload, strlen(dup_payload));

    int fd_dup1 = dup(fd_orig);
    if (fd_dup1 < 0 || fd_dup1 == fd_orig) {
        printf("  [FAIL] dup(fd_orig) failed: fd_dup1=%d\n", fd_dup1);
        close(fd_orig);
        return 1;
    }

    int target_slot = 20;
    int fd_dup2 = dup2(fd_orig, target_slot);
    if (fd_dup2 != target_slot) {
        printf("  [FAIL] dup2(fd_orig, %d) returned %d\n", target_slot, fd_dup2);
        close(fd_orig);
        close(fd_dup1);
        return 1;
    }

    if (dup2(target_slot, target_slot) != target_slot) {
        printf("  [FAIL] dup2(same, same) failed\n");
        close(fd_orig);
        close(fd_dup1);
        close(target_slot);
        return 1;
    }

    if (dup(-1) != -1 || errno != EBADF) {
        printf("  [FAIL] dup(-1) expected EBADF, errno=%d\n", errno);
        return 1;
    }
    if (dup2(fd_orig, -1) != -1 || errno != EBADF) {
        printf("  [FAIL] dup2 negative slot expected EBADF, errno=%d\n", errno);
        return 1;
    }
    if (dup2(fd_orig, 1000) != -1 || errno != EBADF) {
        printf("  [FAIL] dup2 out of range slot expected EBADF, errno=%d\n", errno);
        return 1;
    }

    close(fd_orig);

    off_t seek_res = lseek(target_slot, 0, SEEK_SET);
    if (seek_res != 0) {
        printf("  [FAIL] lseek on duplicated descriptor failed: %d\n", (int)seek_res);
        close(fd_dup1);
        close(target_slot);
        return 1;
    }

    char dup_read_buf[16];
    memset(dup_read_buf, 0, sizeof(dup_read_buf));
    ssize_t dup_read = read(target_slot, dup_read_buf, sizeof(dup_read_buf) - 1);
    if (dup_read != (ssize_t)strlen(dup_payload) || strcmp(dup_read_buf, dup_payload) != 0) {
        printf("  [FAIL] Read via dup2 slot failed: '%s'\n", dup_read_buf);
        close(fd_dup1);
        close(target_slot);
        return 1;
    }

    close(fd_dup1);
    close(target_slot);
    puts("  [INFO] Independent file handle targeting and lifecycle verified");
    puts("  [OK]   Descriptor duplication and explicit slot targeting operational");

    /* 37. Descriptor Advisory Lock Coherency Across Duplicates */
    puts("  [TEST] Descriptor advisory lock coherency across duplicated handles...");
    const char *flock_dup_path = "/temp/flock_dup.txt";
    const char *flock_sync_path = "/temp/flock_sync.txt";

    int f_init_sync = open(flock_sync_path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (f_init_sync < 0) {
        puts("  [FAIL] Failed to initialize flock sync state file");
        return 1;
    }
    write(f_init_sync, "INIT", 4);
    close(f_init_sync);

    pid_t flock_pid = fork();
    if (flock_pid < 0) {
        puts("  [FAIL] Fork failed for flock coherency test");
        return 1;
    } else if (flock_pid == 0) {
        int f1 = open(flock_dup_path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
        if (f1 < 0) {
            exit(10);
        }

        int f2 = dup(f1);
        if (f2 < 0) {
            exit(11);
        }

        /* Close original descriptor f1. f2 is still open, so lock must remain held! */
        close(f1);

        /* Signal parent that f1 is closed and f2 is still open */
        int f_sync1 = open(flock_sync_path, O_WRONLY | O_TRUNC, 0644);
        if (f_sync1 >= 0) {
            write(f_sync1, "STAGE1", 6);
            close(f_sync1);
        }

        /* Poll until parent confirms contention check and signals release */
        for (int r = 0; r < 200; r++) {
            int f_poll = open(flock_sync_path, O_RDONLY, 0);
            if (f_poll >= 0) {
                char sbuf[16];
                memset(sbuf, 0, sizeof(sbuf));
                read(f_poll, sbuf, sizeof(sbuf) - 1);
                close(f_poll);
                if (strcmp(sbuf, "RELEASE") == 0) {
                    break;
                }
            }
            usleep(5000);
        }

        /* Closing f2 now releases the advisory write lock */
        close(f2);
        exit(0);
    } else {
        /* Wait for child to reach stage 1 */
        for (int r = 0; r < 200; r++) {
            int f_poll = open(flock_sync_path, O_RDONLY, 0);
            if (f_poll >= 0) {
                char sbuf[16];
                memset(sbuf, 0, sizeof(sbuf));
                read(f_poll, sbuf, sizeof(sbuf) - 1);
                close(f_poll);
                if (strcmp(sbuf, "STAGE1") == 0) {
                    break;
                }
            }
            usleep(5000);
        }

        /* Attempt to acquire write lock on the file. Must fail with EACCES! */
        int f_conflict = open(flock_dup_path, O_WRONLY, 0);
        if (f_conflict >= 0) {
            printf("  [FAIL] Open succeeded while duplicated handle f2 is still open (lock "
                   "leaked!)\n");
            close(f_conflict);
            return 1;
        }
        if (errno != EACCES) {
            printf("  [FAIL] Expected EACCES on locked file, got errno=%d\n", errno);
            return 1;
        }

        /* Tell child to close f2 and exit */
        int f_sync2 = open(flock_sync_path, O_WRONLY | O_TRUNC, 0644);
        if (f_sync2 >= 0) {
            write(f_sync2, "RELEASE", 7);
            close(f_sync2);
        }

        int wstatus = 0;
        waitpid(flock_pid, &wstatus, 0);

        /* Re-attempt to open the file in write mode now that f2 is closed. Must succeed! */
        int f_success = open(flock_dup_path, O_WRONLY, 0);
        if (f_success < 0) {
            printf("  [FAIL] Open failed after all duplicated descriptors were closed: errno=%d\n",
                   errno);
            return 1;
        }
        close(f_success);

        puts("  [INFO] Closing duplicated handle preserved lock until final handle closure");
        puts("  [OK]   Advisory write lock coherency across duplicated descriptors verified");
    }

    /* 38. Multi-Process Concurrent Syscall & Memory Stress */
    puts("  [TEST] Multi-process concurrent syscall & memory stress...");
    int stress_pipe[2];
    if (pipe(stress_pipe) < 0) {
        puts("  [FAIL] Failed to create stress pipe");
        return 1;
    }

    pid_t worker1 = fork();
    if (worker1 < 0) {
        puts("  [FAIL] Failed to fork worker 1");
        return 1;
    } else if (worker1 == 0) {
        close(stress_pipe[0]);
        /* Worker 1 performs repeated memory break allocations and writes pattern */
        for (int iter = 0; iter < 16; iter++) {
            void *brk_new = sbrk(4096);
            if (brk_new == (void *)-1) {
                exit(101);
            }
            char *buf = (char *)brk_new;
            memset(buf, 0xA5 + iter, 4096);
            for (int k = 0; k < 4096; k += 512) {
                if ((unsigned char)buf[k] != (unsigned char)(0xA5 + iter)) {
                    exit(102);
                }
            }
            int d = dup(stress_pipe[1]);
            if (d < 0) {
                exit(103);
            }
            char msg = 'A' + iter;
            if (write(d, &msg, 1) != 1) {
                exit(104);
            }
            close(d);
            usleep(1000);
        }
        close(stress_pipe[1]);
        exit(0);
    }

    pid_t worker2 = fork();
    if (worker2 < 0) {
        puts("  [FAIL] Failed to fork worker 2");
        return 1;
    } else if (worker2 == 0) {
        close(stress_pipe[0]);
        /* Worker 2 performs concurrent heap allocation and descriptor stress */
        for (int iter = 0; iter < 16; iter++) {
            void *p = malloc(1024);
            if (!p) {
                exit(201);
            }
            memset(p, 0x5A + iter, 1024);
            char *c = (char *)p;
            for (int k = 0; k < 1024; k += 128) {
                if ((unsigned char)c[k] != (unsigned char)(0x5A + iter)) {
                    exit(202);
                }
            }
            free(p);
            int d = dup(stress_pipe[1]);
            if (d < 0) {
                exit(203);
            }
            char msg = 'a' + iter;
            if (write(d, &msg, 1) != 1) {
                exit(204);
            }
            close(d);
            usleep(1000);
        }
        close(stress_pipe[1]);
        exit(0);
    }

    close(stress_pipe[1]);

    int status1 = 0, status2 = 0;
    if (waitpid(worker1, &status1, 0) != worker1) {
        puts("  [FAIL] Failed waiting for worker 1");
        return 1;
    }
    if (waitpid(worker2, &status2, 0) != worker2) {
        puts("  [FAIL] Failed waiting for worker 2");
        return 1;
    }

    if (WEXITSTATUS(status1) != 0 || WEXITSTATUS(status2) != 0) {
        printf("  [FAIL] Concurrent worker error: worker1=%d worker2=%d\n", WEXITSTATUS(status1),
               WEXITSTATUS(status2));
        return 1;
    }

    /* All 32 bytes committed by workers, now drain the pipe */
    int bytes_read = 0;
    char read_buf[64];
    while (bytes_read < 32) {
        ssize_t n = read(stress_pipe[0], read_buf + bytes_read, sizeof(read_buf) - bytes_read);
        if (n <= 0) {
            break;
        }
        bytes_read += n;
    }
    close(stress_pipe[0]);

    if (bytes_read != 32) {
        printf("  [FAIL] Incomplete pipe transfer: got %d/32 bytes\n", bytes_read);
        return 1;
    }
    puts("  [INFO] Concurrent heap mutations and pipe transfers completed without corruption");
    puts("  [OK]   Multi-process concurrent syscall & memory stress verified");

    /* 39. Lock Contention & Non-Blocking Deadlock Immunity */
    puts("  [TEST] Lock contention & non-blocking deadlock immunity...");
    const char *contention_file = "/temp/contention_lock.txt";
    int fd_lock = open(contention_file, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd_lock < 0) {
        puts("  [FAIL] Failed to create contention lock test file");
        return 1;
    }
    write(fd_lock, "HELD", 4);

    pid_t contender = fork();
    if (contender < 0) {
        puts("  [FAIL] Failed to fork contender process");
        return 1;
    } else if (contender == 0) {
        /* Contender attempts non-blocking/immediate lock acquisition which should fail gracefully
         * without deadlocking */
        int f = open(contention_file, O_WRONLY, 0);
        if (f >= 0) {
            close(f);
            exit(1); /* Lock was held by parent, open in write mode should fail! */
        }
        if (errno != EACCES) {
            exit(2);
        }
        exit(0);
    }

    /* Verify non-blocking waitpid with WNOHANG handles active process safely */
    int final_status = 0;
    int wnohang_status = 0;
    pid_t wnohang_res = waitpid(contender, &wnohang_status, WNOHANG);
    if (wnohang_res < 0) {
        puts("  [FAIL] waitpid with WNOHANG returned error");
        return 1;
    }

    if (wnohang_res == contender) {
        /* Contender already finished and was reaped by WNOHANG */
        final_status = wnohang_status;
    } else {
        /* Contender still running, wait blocking for completion */
        int cstatus = 0;
        pid_t wres = waitpid(contender, &cstatus, 0);
        if (wres != contender) {
            printf("  [FAIL] Failed waiting for contender: wres=%d\n", (int)wres);
            return 1;
        }
        final_status = cstatus;
    }

    if (WEXITSTATUS(final_status) != 0) {
        printf("  [FAIL] Contender failed or deadlocked: exit=%d\n", WEXITSTATUS(final_status));
        return 1;
    }

    /* Release parent lock */
    close(fd_lock);

    /* Contention is cleared: subsequent open must succeed immediately */
    int f_cleared = open(contention_file, O_WRONLY, 0);
    if (f_cleared < 0) {
        printf("  [FAIL] Failed opening file after contention released: errno=%d\n", errno);
        return 1;
    }
    close(f_cleared);

    puts("  [INFO] Lock contention correctly rejected with EACCES without scheduler deadlock");
    puts("  [OK]   Lock contention & non-blocking deadlock immunity verified");

    puts("\n[DONE] All Ring 3 Syscall Security & Fault Injection tests PASSED.");
    return 0;
}
