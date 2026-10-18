/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Netfilter Firewall Utility (iptables)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

#define IPTABLES_RULES_PATH "/proc/net/iptables"
#define IPTABLES_NAMES_PATH "/proc/net/ip_tables_names"
#define NF_CONNTRACK_PATH "/proc/net/stat/nf_conntrack"

static void print_usage(void) {
    puts("Usage: iptables [options]");
    puts("");
    puts("Commands:");
    puts("  -L, --list [chain]        List active rules in chains");
    puts("  -S, --list-rules [chain]  Print rules in specification format");
    puts("  -F, --flush [chain]       Flush all rules and conntrack entries");
    puts("  -A, --append <chain>      Append a new rule to chain");
    puts("  -D, --delete <chain> <n>  Delete rule number n from chain");
    puts("  --status                  Show in-kernel packet inspection telemetry");
    puts("  -h, --help                Display this help message");
    puts("");
    puts("Rule options:");
    puts("  -p, --proto <protocol>    Protocol (tcp, udp, icmp, any)");
    puts("  -s, --source <ip/mask>    Source IP/mask (default: 0.0.0.0/0)");
    puts("  -d, --destination <ip>    Destination IP/mask (default: 0.0.0.0/0)");
    puts("  --dport <port>            Destination TCP/UDP port (0 = any)");
    puts("  -j, --jump <target>       Action target (ACCEPT, DROP, REJECT)");
    puts("  -t, --table <table_name>  Table to operate on (default: filter)");
}

static int check_table_valid(const char *table) {
    FILE *f = fopen(IPTABLES_NAMES_PATH, "r");
    if (!f) {
        return 1;
    }
    char line[64];
    int found = 0;
    while (fgets(line, sizeof(line), f)) {
        size_t len = strlen(line);
        if (len > 0 && line[len - 1] == '\n') {
            line[len - 1] = '\0';
        }
        if (strcmp(line, table) == 0) {
            found = 1;
            break;
        }
    }
    fclose(f);
    return found;
}

static int list_rules_formatted(const char *target_chain) {
    FILE *f = fopen(IPTABLES_RULES_PATH, "r");
    if (!f) {
        puts("iptables: failed to open /proc/net/iptables");
        return 1;
    }

    char line[256];
    char current_chain[32] = "";
    int header_printed = 0;

    while (fgets(line, sizeof(line), f)) {
        if (line[0] == '#' || line[0] == '*' || strncmp(line, "COMMIT", 6) == 0) {
            continue;
        }

        if (line[0] == ':') {
            char chain_name[32] = "";
            char policy[32] = "ACCEPT";
            sscanf(line, ":%31s %31s", chain_name, policy);

            if (!target_chain || strcmp(target_chain, chain_name) == 0) {
                if (header_printed) {
                    puts("");
                }
                printf("Chain %s (policy %s)\n", chain_name, policy);
                puts("target     prot opt source               destination         ");
                header_printed = 1;
                strncpy(current_chain, chain_name, sizeof(current_chain) - 1);
            } else {
                current_chain[0] = '\0';
            }
            continue;
        }

        if (strncmp(line, "-A ", 3) == 0) {
            char chain[32] = "";
            char proto[16] = "all";
            char src[32] = "0.0.0.0/0";
            char dst[32] = "0.0.0.0/0";
            char target[16] = "ACCEPT";
            int dport = 0;

            char *p = line + 3;
            if (sscanf(p, "%31s", chain) != 1) {
                continue;
            }

            if (target_chain && strcmp(target_chain, chain) != 0) {
                continue;
            }

            char *tok = strstr(line, "-p ");
            if (tok) {
                sscanf(tok + 3, "%15s", proto);
            }
            tok = strstr(line, "-s ");
            if (tok) {
                sscanf(tok + 3, "%31s", src);
            }
            tok = strstr(line, "-d ");
            if (tok) {
                sscanf(tok + 3, "%31s", dst);
            }
            tok = strstr(line, "--dport ");
            if (tok) {
                sscanf(tok + 8, "%d", &dport);
            }
            tok = strstr(line, "-j ");
            if (tok) {
                sscanf(tok + 3, "%15s", target);
            }

            char proto_lower[16];
            for (size_t i = 0; i <= strlen(proto); i++) {
                char c = proto[i];
                if (c >= 'A' && c <= 'Z') {
                    c = (char)(c + 32);
                }
                proto_lower[i] = c;
            }

            if (dport > 0) {
                char opt_str[32];
                snprintf(opt_str, sizeof(opt_str), "%s dpt:%d", proto_lower, dport);
                printf("%-10s %-4s --  %-20s %-20s %s\n", target, proto_lower, src, dst, opt_str);
            } else {
                printf("%-10s %-4s --  %-20s %-20s\n", target, proto_lower, src, dst);
            }
        }
    }

    fclose(f);
    return 0;
}

static int list_rules_specs(const char *target_chain) {
    FILE *f = fopen(IPTABLES_RULES_PATH, "r");
    if (!f) {
        puts("iptables: failed to open /proc/net/iptables");
        return 1;
    }

    char line[256];
    while (fgets(line, sizeof(line), f)) {
        if (line[0] == '#' || line[0] == '*' || strncmp(line, "COMMIT", 6) == 0) {
            continue;
        }

        if (line[0] == ':') {
            char chain[32] = "";
            char policy[32] = "ACCEPT";
            sscanf(line, ":%31s %31s", chain, policy);
            if (!target_chain || strcmp(target_chain, chain) == 0) {
                printf("-P %s %s\n", chain, policy);
            }
            continue;
        }

        if (strncmp(line, "-A ", 3) == 0) {
            char chain[32] = "";
            sscanf(line + 3, "%31s", chain);
            if (!target_chain || strcmp(target_chain, chain) == 0) {
                size_t len = strlen(line);
                if (len > 0 && line[len - 1] == '\n') {
                    line[len - 1] = '\0';
                }
                char *cnt = strstr(line, " -c ");
                if (cnt) {
                    *cnt = '\0';
                }
                puts(line);
            }
        }
    }

    fclose(f);
    return 0;
}

static int show_status(void) {
    puts("Keira Netfilter Kernel Telemetry:");
    long res = sys_netfilter(NETFILTER_CMD_STATUS, 0, 0);
    if (res < 0) {
        puts("iptables: SYS_NETFILTER returned error.");
    }

    FILE *f = fopen(NF_CONNTRACK_PATH, "r");
    if (f) {
        puts("\nConnection Tracking Statistics (/proc/net/stat/nf_conntrack):");
        char line[256];
        while (fgets(line, sizeof(line), f)) {
            printf("%s", line);
        }
        fclose(f);
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        return list_rules_formatted(NULL);
    }

    const char *table = "filter";
    int mode_list = 0;
    int mode_specs = 0;
    int mode_flush = 0;
    int mode_append = 0;
    int mode_delete = 0;
    int mode_status = 0;

    const char *chain = NULL;
    const char *proto = "TCP";
    const char *src = "0.0.0.0/0";
    const char *dst = "0.0.0.0/0";
    int dport = 0;
    const char *target = "ACCEPT";
    int rule_num = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];

        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-t") == 0 || strcmp(arg, "--table") == 0) {
            if (i + 1 < argc) {
                table = argv[++i];
            }
        } else if (strcmp(arg, "-L") == 0 || strcmp(arg, "--list") == 0) {
            mode_list = 1;
            if (i + 1 < argc && argv[i + 1][0] != '-') {
                chain = argv[++i];
            }
        } else if (strcmp(arg, "-S") == 0 || strcmp(arg, "--list-rules") == 0) {
            mode_specs = 1;
            if (i + 1 < argc && argv[i + 1][0] != '-') {
                chain = argv[++i];
            }
        } else if (strcmp(arg, "-F") == 0 || strcmp(arg, "--flush") == 0) {
            mode_flush = 1;
            if (i + 1 < argc && argv[i + 1][0] != '-') {
                chain = argv[++i];
            }
        } else if (strcmp(arg, "-A") == 0 || strcmp(arg, "--append") == 0) {
            mode_append = 1;
            if (i + 1 < argc) {
                chain = argv[++i];
            }
        } else if (strcmp(arg, "-D") == 0 || strcmp(arg, "--delete") == 0) {
            mode_delete = 1;
            if (i + 1 < argc) {
                chain = argv[++i];
            }
            if (i + 1 < argc && argv[i + 1][0] != '-') {
                rule_num = atoi(argv[++i]);
            }
        } else if (strcmp(arg, "--status") == 0) {
            mode_status = 1;
        } else if (strcmp(arg, "-p") == 0 || strcmp(arg, "--proto") == 0) {
            if (i + 1 < argc) {
                proto = argv[++i];
            }
        } else if (strcmp(arg, "-s") == 0 || strcmp(arg, "--source") == 0) {
            if (i + 1 < argc) {
                src = argv[++i];
            }
        } else if (strcmp(arg, "-d") == 0 || strcmp(arg, "--destination") == 0) {
            if (i + 1 < argc) {
                dst = argv[++i];
            }
        } else if (strcmp(arg, "--dport") == 0) {
            if (i + 1 < argc) {
                dport = atoi(argv[++i]);
            }
        } else if (strcmp(arg, "-j") == 0 || strcmp(arg, "--jump") == 0) {
            if (i + 1 < argc) {
                target = argv[++i];
            }
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("iptables: unknown option \"%s\"\n", arg);
            puts("Try 'iptables --help' for more information.");
            return 1;
        }
    }

    if (!check_table_valid(table)) {
        printf("iptables: Table does not exist (see /proc/net/ip_tables_names)\n");
        return 1;
    }

    if (mode_status) {
        return show_status();
    }

    if (mode_flush) {
        long res = sys_netfilter(NETFILTER_CMD_FLUSH, 0, 0);
        if (res >= 0) {
            printf("iptables: Flushed rules in table `%s`.\n", table);
            return 0;
        } else {
            puts("iptables: Failed to flush rules.");
            return 1;
        }
    }

    if (mode_delete) {
        if (!chain || rule_num <= 0) {
            puts("iptables: -D requires <chain> and rule number (> 0)");
            return 1;
        }
        long res = sys_netfilter(NETFILTER_CMD_DEL_RULE, (uint64_t)rule_num, 0);
        if (res >= 0) {
            printf("iptables: Deleted rule %d from chain %s.\n", rule_num, chain);
            return 0;
        } else {
            printf("iptables: Failed to delete rule %d (out of range or not found).\n", rule_num);
            return 1;
        }
    }

    if (mode_append) {
        if (!chain) {
            puts("iptables: -A requires <chain>");
            return 1;
        }
        char rule_spec[128];
        snprintf(rule_spec, sizeof(rule_spec), "%s %s %d %s %s %s", chain, proto, dport, target,
                 src, dst);

        long res = sys_netfilter(NETFILTER_CMD_ADD_RULE, (uint64_t)(uintptr_t)rule_spec, 0);
        if (res > 0) {
            printf("iptables: Rule %ld added to %s chain.\n", res, chain);
            return 0;
        } else {
            puts("iptables: Failed to append rule (table full or invalid spec).");
            return 1;
        }
    }

    if (mode_specs) {
        return list_rules_specs(chain);
    }

    if (mode_list || (!mode_specs && !mode_flush && !mode_append && !mode_delete)) {
        return list_rules_formatted(chain);
    }

    return 0;
}
