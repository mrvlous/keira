#!/bin/sh
# SPDX-License-Identifier: GPL-2.0-only

echo "[INIT] Initializing canonical userland system environment..."
hostname keira
mount -t tmpfs none /tmp
chmod 1777 /tmp
cat /usr/src/main.c > /tmp/main.c
mkdir -p /var/run /var/log
ifconfig lo
uname -a
echo "[INIT] Userland system initialization sequence completed."
