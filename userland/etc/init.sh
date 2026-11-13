#!/bin/sh
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.

echo "[INIT] Initializing canonical userland system environment..."

# Configure network node hostname
hostname keira

# Mount ephemeral in-memory scratch volume on /tmp and apply sticky permissions
mount -t tmpfs none /tmp
chmod 1777 /tmp

# Stage default C compiler demonstration source
cat /usr/src/main.c > /tmp/main.c

# Create standard runtime tracking directories
mkdir -p /var/run /var/log

# Initialize loopback network interface
ifconfig lo

# Display system identification banner
uname -a

echo "[INIT] Userland system initialization sequence completed."
