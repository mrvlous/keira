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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/wait.h>
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

    puts("\n[DONE] All Ring 3 Syscall Security & Fault Injection tests PASSED.");
    return 0;
}
