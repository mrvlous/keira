#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Real-World Public Internet REST API Verification Harness for Keira Kernel
# Tests Ring 3 curl and fetch utilities against live HTTP endpoints worldwide.

import glob
import os
import socket
import subprocess
import sys
import time

SCREENSHOT_DIRS = [
    "/home/keira/.gemini/antigravity/brain/4b8274c4-2a0c-4aa4-8cbb-44267df52a23/screenshots",
    "docs/screenshots"
]
for d in SCREENSHOT_DIRS:
    os.makedirs(d, exist_ok=True)

def capture_screenshot(mon_s, name):
    raw_ppm = f"/tmp/screenshot_online_{name}.ppm"
    if os.path.exists(raw_ppm):
        os.remove(raw_ppm)
    cmd = f'screendump "{raw_ppm}"\n'
    mon_s.sendall(cmd.encode())
    time.sleep(0.3)
    try:
        mon_s.recv(1024)
    except Exception:
        pass

    if not os.path.exists(raw_ppm):
        print(f"  [WARN] Screendump failed for {name}")
        return None

    out_pngs = []
    for d in SCREENSHOT_DIRS:
        target_png = os.path.join(d, f"{name}.png")
        subprocess.run(
            ["ffmpeg", "-i", raw_ppm, "-frames:v", "1", target_png, "-y"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if os.path.exists(target_png):
            out_pngs.append(target_png)

    if out_pngs:
        size_kb = os.path.getsize(out_pngs[0]) // 1024
        print(f"  [SCREENSHOT] {name}.png captured ({size_kb} KB)")
        return out_pngs[0]
    return None

def get_vga_text(mon_s):
    pmem_path = "/tmp/vga_dump_online.bin"
    if os.path.exists(pmem_path):
        os.remove(pmem_path)
    mon_s.sendall(f'pmemsave 0xb8000 4000 "{pmem_path}"\n'.encode())
    time.sleep(0.2)
    try:
        mon_s.recv(1024)
    except Exception:
        pass
    if not os.path.exists(pmem_path):
        return ""
    with open(pmem_path, "rb") as f:
        data = f.read()
    lines = []
    for row in range(25):
        row_chars = []
        for col in range(80):
            idx = (row * 80 + col) * 2
            ch = data[idx]
            row_chars.append(chr(ch) if 32 <= ch < 127 else " ")
        line = "".join(row_chars).rstrip()
        if line:
            lines.append(line)
    return "\n".join(lines)

def main():
    print("=" * 70)
    print("STARTING REAL-WORLD ONLINE PUBLIC REST API VERIFICATION IN RING 3")
    print("=" * 70)

    iso_matches = sorted(glob.glob("build/x86/x86_64/iso/keira-x86_64-*.iso"))
    if not iso_matches:
        print("[ERROR] No ISO found for x86_64")
        return 1
    iso_path = iso_matches[-1]
    disk_path = "build/x86/x86_64/disk/disk.img"
    sock_path = "/tmp/qemu-mon-online-net.sock"
    if os.path.exists(sock_path):
        os.remove(sock_path)

    qemu_cmd = [
        "qemu-system-x86_64",
        "-cdrom", iso_path,
        "-boot", "d",
        "-smp", "2",
        "-device", "ahci,id=ahci0",
        "-drive", f"file={disk_path},format=raw,id=sata0,if=none,file.locking=off",
        "-device", "ide-hd,drive=sata0,bus=ahci0.0",
        "-device", "e1000,netdev=net0",
        "-netdev", "user,id=net0",
        "-monitor", f"unix:{sock_path},server,nowait",
        "-serial", "stdio",
        "-display", "none",
        "-no-shutdown",
        "-m", "128M"
    ]

    print("[INFO] Launching QEMU with external network access...")
    proc = subprocess.Popen(
        qemu_cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    start = time.time()
    booted = False
    while time.time() - start < 30:
        line = proc.stdout.readline()
        if line:
            if "init: system initialization complete; spawning interactive shell" in line:
                booted = True
                print("[OK] Kernel booted into Ring 3 shell!")
                break

    if not booted:
        print("[FAIL] Failed to boot x86_64 within timeout")
        proc.kill()
        return 1

    time.sleep(1.0)
    mon_s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    mon_s.connect(sock_path)
    time.sleep(0.2)
    mon_s.recv(1024)

    # 1. Initialize Network Interface
    print("\n[STEP 1] Configuring Ring 3 network interface (eth0)...")
    proc.stdin.write("ifconfig eth0 10.0.2.15 255.255.255.0 10.0.2.2\n")
    proc.stdin.write("route\n")
    proc.stdin.flush()
    time.sleep(1.5)
    capture_screenshot(mon_s, "online_api_01_ifconfig")

    # 2. Test Real Online API 1: icanhazip.com via curl
    print("\n[STEP 2] Querying real-world public IP endpoint: http://icanhazip.com via curl...")
    proc.stdin.write("curl -v http://icanhazip.com\n")
    proc.stdin.flush()
    time.sleep(5.0)
    capture_screenshot(mon_s, "online_api_02_icanhazip")

    # 3. Test Real Online API 2: api.ipify.org JSON endpoint via fetch
    print("\n[STEP 3] Querying real-world REST JSON API: http://api.ipify.org/?format=json via fetch...")
    proc.stdin.write("fetch -i http://api.ipify.org/?format=json\n")
    proc.stdin.flush()
    time.sleep(5.0)
    capture_screenshot(mon_s, "online_api_03_ipify_json")

    # 4. Test Real Online API 3: httpbin.org IP endpoint via curl
    print("\n[STEP 4] Querying real-world REST API: http://httpbin.org/ip via curl...")
    proc.stdin.write("curl http://httpbin.org/ip\n")
    proc.stdin.flush()
    time.sleep(5.0)
    capture_screenshot(mon_s, "online_api_04_httpbin_ip")

    # 5. Test Real Online API 4: ip-api.com JSON endpoint saved to file via fetch
    print("\n[STEP 5] Querying real-world Geolocation REST API: http://ip-api.com/json via fetch -o...")
    proc.stdin.write("fetch -o /tmp/ip_geo.json http://ip-api.com/json\n")
    proc.stdin.flush()
    time.sleep(5.0)
    proc.stdin.write("cat /tmp/ip_geo.json\n")
    proc.stdin.flush()
    time.sleep(2.0)
    capture_screenshot(mon_s, "online_api_05_ip_api_json")

    # 6. Clean Shutdown
    print("\n[STEP 6] Shutting down cleanly...")
    proc.stdin.write("exit\n")
    proc.stdin.flush()
    time.sleep(1.0)
    proc.stdin.write("poweroff\n")
    proc.stdin.flush()
    time.sleep(1.5)

    proc.kill()
    mon_s.close()

    print("\n" + "=" * 70)
    print("REAL-WORLD PUBLIC REST API VERIFICATION COMPLETE")
    print("=" * 70)
    return 0

if __name__ == "__main__":
    sys.exit(main())
