<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Security & Authentication Shell Commands

This document details all 7 native built-in commands in Keira Kernel related to multi-user authentication, privilege delegation, Seccomp system call sandboxes, Extended Berkeley Packet Filter (eBPF) runtime virtual machine, Trusted Platform Module (TPM 2.0), and Mandatory Access Control (MAC) Type Enforcement policies.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `login` | `login [username]` | `[Active]` | Authenticate user credentials against `/config/sys/passwd` and switch session |
| `user` | `user [whoami \| list \| add <name> \| del <name>]` | `[Active]` | Manage system user identities, UID/GID mapping, and home folders in `/users/` |
| `protect` | `protect <path> [readonly \| hidden \| archive]` | `[Active]` | Configure FAT filesystem attributes and access permissions on files |
| `seccomp` | `seccomp [status \| strict \| filter \| allow <num> \| deny <num> \| reset]` | `[Active]` | Inspect and configure Secure Computing system call sandbox filters (Syscall 52) |
| `bpf` | `bpf [status \| list \| maps \| test \| map-get <id> <k> \| map-set <id> <k> <v>]` | `[Active]` | Inspect and execute Extended Berkeley Packet Filter programs and maps (Syscall 78) |
| `tpm` | `tpm [status \| pcr [idx] \| extend <idx> <data> \| seal [mask] <secret> \| unseal \| log \| quote [mask] \| test]` | `[Active]` | Query, extend, seal, and test Trusted Platform Module (TPM 2.0) enclave (Syscall 79) |
| `mac` | `mac [status \| enforce \| permissive \| disable \| rules \| audit \| test]` | `[Active]` | Inspect and configure Mandatory Access Control Type Enforcement policies |

---

## Detailed Usage

### `seccomp`
Inspects and configures the in-kernel Secure Computing (Seccomp) system call sandboxing engine:
```bash
keira> seccomp status
Secure Computing (Seccomp) Sandbox Status:
  Operational Mode  : Disabled (Syscalls Unrestricted)
  Syscalls Inspected: 142
  Blocked Violations: 0
  Enforcement Hook  : Syscall Dispatcher (Syscall 52)

keira> seccomp strict
[OK] Seccomp Strict Sandbox enabled.
     Only read (7, 15), write (1, 8, 16), exit (2), sigreturn (65), and seccomp (52) allowed.

keira> seccomp filter
[OK] Seccomp Filter Sandbox enabled (Bitmask-based filtering).

keira> seccomp deny 21
[OK] Syscall #21 removed from allowed whitelist mask.

keira> seccomp allow 21
[OK] Syscall #21 added to allowed whitelist mask.

keira> seccomp reset
[OK] Seccomp sandbox reset and set to Disabled.
```

---

### `bpf`
Manages the in-kernel Extended Berkeley Packet Filter (eBPF) runtime virtual machine, bytecode verifier, and in-kernel maps:
```bash
keira> bpf status
Extended Berkeley Packet Filter (eBPF) Subsystem:
  VM Engine State   : Active (In-Kernel Bytecode Interpreter)
  In-Kernel Verifier: Enforced (CFG Bounded-Cycle Safety Check)
  Loaded Programs   : 1
  Active Maps       : 2
  Total Executions  : 48
  Syscall Interface : Syscall 78 (SYS_BPF)

keira> bpf list
Loaded eBPF Kernel Programs:
  ID  TYPE           NAME            INSNS  RUNS   DROPS  PASSES
  --  -------------  --------------  -----  -----  -----  ------
  1   socket_filter  http_filter     6      48     12     36

keira> bpf maps
Active eBPF In-Kernel Maps:
  ID  TYPE         NAME             KEY  VAL  CAPACITY  ENTRIES
  --  -----------  ---------------  ---  ---  --------  -------
  1   array        drop_counters    4B   8B   16        0
  2   hash         port_blacklist   4B   8B   16        2

keira> bpf test
Running in-kernel eBPF verification and VM test suite:
  [1/3] Static Verifier Analysis ... PASSED (6 insns, 0 jumps OOB, termination guaranteed)
  [2/3] VM Execute Packet #1 (TCP Port 80) ... ACCEPTED (Return: 0x0000FFFF)
  [3/3] VM Execute Packet #2 (UDP Port 53) ... FILTERED / DROPPED (Correct: Non-TCP)
eBPF VM Interpreter & Verifier validation completed successfully.
```

---

### `tpm`
Interacts with the bare-metal Trusted Platform Module (TPM 2.0) hardware security enclave, 24 SHA-256 Platform Configuration Registers (PCRs), measured boot event logs, and sealed storage:
```bash
keira> tpm status
TPM 2.0 Hardware Security Enclave Status:
  Device State      : Active (TCG TPM 2.0 Specification rev 01.59)
  Locality Base     : 0x00000000FED40000 (Locality 0 MMIO Enclave)
  Hardware Probe    : VID 0x1B36 DID 0x0001 [CONNECTED]
  Active PCR Banks  : SHA-256 (24 Registers [PCR 0..23])
  Total Measurements: 6
  Event Log Records : 6
  Syscall Interface : Syscall 79 (SYS_TPM2)

keira> tpm pcr 0
PCR[00] (SHA-256): fa070e143818114446485b56111b83f5bdc518b9129d2d4a25a83930c2d5831c (Firmware / BIOS IVT)

keira> tpm log
TPM 2.0 Measured Boot & Integrity Event Log:
  PCR  TYPE        EVENT NAME           SHA-256 DIGEST (PREFIX)
  ---  ----------  -------------------  -----------------------
  [00] 0x00000001  BIOS_IVT_MEASURED    fa070e1438181144...
  [01] 0x0000000A  PLATFORM_CONFIG      d282f6092d364274...
  [02] 0x00000005  HOST_BUS_TOPOLOGY    74b8a44bd0a650e0...
  [07] 0x00000004  SECUREBOOT_POLICY    24f2d8a2849acbd7...
  [04] 0x00000005  KERNEL_TEXT_SEGMENT  7816d62ec7be5a8a...
  [05] 0x00000005  INITRD_ARCHIVE       3ea86060ee0b0b97...

keira> tpm seal 0x000000FF "DATABASE_MASTER_SECRET"
[OK] Secret sealed successfully under PCR mask 0x000000FF
  Expected Quote: 4f5e71ba099c2d15be8701aa34e912429f0322bca678ef40c99b8214fa39b02a
  AES-GCM AuthTag: 2a8f90c1e457bb92019488aef402319c
  Ciphertext Len: 22 bytes

keira> tpm unseal
[OK] Secret unsealed successfully (PCR policy attestation verified):
  Plaintext: DATABASE_MASTER_SECRET

keira> tpm test
Running Bare-Metal TPM 2.0 Security Subsystem Selftest:
  [1/4] Generating Attestation Quote (PCR 0,1,4): OK
  [2/4] Sealing Data under PCR Mask 0x00000013: OK
  [3/4] Unsealing Secret (Authentic State): OK (Verified)
  [4/4] Tamper Detection (PCR Extension & Rejection): OK (Correctly Rejected)

[PASS] All TPM 2.0 Hardware Security Assertions Verified Successfully.
```

---

### `mac`
Inspects and configures the Mandatory Access Control (MAC) Type Enforcement security policy matrix and audits access decisions:
```bash
keira> mac status
Mandatory Access Control (MAC) Subsystem:
  Policy Engine     : Type Enforcement Active
  Operational Mode  : Permissive (Auditing Violations without Blocking)
  Active Policy Rules: 10
  Paths Evaluated   : 84
  Policy Violations : 1
  Security Domains  : kernel, system, user, guest, network

keira> mac rules
Mandatory Access Control Type Enforcement Policy Matrix:
  #   DOMAIN   TARGET PATH PREFIX        PERMISSIONS
  --  -------  ------------------------  -----------
  0   kernel   /                         R W X A
  1   system   /system/                  R W X A
  2   system   /config/                  R W A
  3   user     /users/                   R W X A
  4   user     /system/bin/              R X
  5   user     /config/                  R
  6   user     /temp/                    R W A
  7   guest    /system/bin/              R X
  8   guest    /temp/                    R W
  9   guest    /config/                  [DENIED / NO ACCESS]

keira> mac test
Running Type Enforcement policy evaluation tests:
  [1/4] User (PID 2) READ on /system/bin/ls ... PASSED (Allowed)
  [2/4] User (PID 2) WRITE on /config/sys/passwd ... PASSED (Blocked as forbidden)
  [3/4] System (PID 1) WRITE on /config/sys/passwd ... PASSED (Allowed)
  [4/4] User (PID 2) EXEC on /users/admin/script.sh ... PASSED (Allowed)
Type Enforcement test run complete. Restored previous operational mode.
```

---

### `user` & `login`
Inspects user credentials and privilege levels:
```bash
keira> user whoami
Current User : admin (UID: 0, GID: 0)
Role         : System Administrator
Home         : /users/admin
Shell        : /system/bin/shell
```

---

### `protect <path>`
Sets read-only or hidden security attributes on a target FAT16 file:
```bash
keira> protect /config/sys/kernel.cfg readonly
[OK] File /config/sys/kernel.cfg attribute updated to Read-Only.
```
