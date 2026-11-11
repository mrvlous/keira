/* SPDX-License-Identifier: GPL-2.0-only */

#ifndef _SYS_REBOOT_H
#define _SYS_REBOOT_H

#define LINUX_REBOOT_MAGIC1 0xfee1dead
#define LINUX_REBOOT_MAGIC2 672274793
#define LINUX_REBOOT_MAGIC2A 85072278
#define LINUX_REBOOT_MAGIC2B 369367448
#define LINUX_REBOOT_MAGIC2C 537993216

#define RB_AUTOBOOT 0x01234567
#define RB_HALT_SYSTEM 0xcdef0123
#define RB_ENABLE_CAD 0x89abcdef
#define RB_DISABLE_CAD 0x00000000
#define RB_POWER_OFF 0x4321fedc
#define RB_SW_SUSPEND 0xd000fce2
#define RB_KEXEC 0x45584543

int reboot(int cmd);
int sys_reboot(int magic1, int magic2, int cmd, void *arg);

#endif /* _SYS_REBOOT_H */
