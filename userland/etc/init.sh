#!/bin/sh
# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Freestanding Kernel from Scratch
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.

echo "[INIT] Initializing canonical userland system environment..."
hostname keira
mount -t tmpfs none /tmp
chmod 1777 /tmp
mkdir -p /var/run /var/log
ifconfig lo
uname -a
echo "[INIT] Userland system initialization sequence completed."
