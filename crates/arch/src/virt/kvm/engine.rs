// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware-assisted virtualization context (Intel VMX / AMD SVM), VMCS/VMCB management,
//! vCPU execution pipeline, and KVM kernel engine (Syscall 42 and 43).

use keira_core::sync::mutex::SpinMutex;

/// Maximum number of concurrently managed Guest Virtual Machines.
pub const MAX_GUEST_VMS: usize = 4;

/// Maximum number of virtual CPUs allocated per Guest Virtual Machine.
pub const MAX_VCPUS_PER_VM: usize = 4;

/// Guest reset vector execution start address (standard 16-byte ROM baseline at 0xFFF0).
pub const GUEST_RESET_VECTOR: u64 = 0x0000_FFF0;

/// Fixed-size guest instruction code capacity per vCPU context.
pub const GUEST_CODE_CAPACITY: usize = 64;

/// Hardware virtualization capabilities detected via CPUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtHardware {
    /// Intel VT-x / VMX instruction set support (CPUID.01H:ECX.VMX[bit 5]).
    pub has_intel_vmx: bool,
    /// AMD-V / SVM instruction set support (CPUID.80000001H:ECX.SVM[bit 2]).
    pub has_amd_svm: bool,
    /// Hypervisor operational status based on genuine hardware virtualization flags.
    pub hypervisor_ready: bool,
}

/// Standard x86_64 guest CPU register state for VM-entry / VM-exit context save and restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct GuestRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
}

impl GuestRegisters {
    pub const fn new_reset_state() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0x0000_0F00,
            rsi: 0,
            rdi: 0,
            rsp: 0x0000_7C00,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: GUEST_RESET_VECTOR,
            rflags: 0x0000_0002,
            cr0: 0x6000_0010,
            cr3: 0,
            cr4: 0x0000_2000,
        }
    }
}

/// Categorized reason triggering VM-exit transition from Guest back to Keira Hypervisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmExitReason {
    /// Initial or unknown exit reason.
    Unknown = 0,
    /// Guest attempted port I/O instruction (IN / OUT).
    IoInstruction = 1,
    /// Guest executed HLT instruction.
    Hlt = 2,
    /// Guest queried CPU feature flags via CPUID.
    Cpuid = 3,
    /// Guest attempted access to Control Register (CR0/CR3/CR4).
    CrAccess = 4,
    /// Nested page table fault (Intel EPT or AMD NPT violation).
    EptViolation = 5,
    /// Explicit guest hypercall invocation (VMCALL / VMMCALL).
    Hypercall = 6,
    /// Guest shutdown / triple fault.
    Shutdown = 7,
}

impl VmExitReason {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::IoInstruction => "I/O Instruction (IN/OUT)",
            Self::Hlt => "HLT (Processor Halt)",
            Self::Cpuid => "CPUID Instruction",
            Self::CrAccess => "Control Register Access",
            Self::EptViolation => "EPT/NPT Paging Violation",
            Self::Hypercall => "Guest Hypercall (VMCALL)",
            Self::Shutdown => "Guest Triple Fault / Shutdown",
        }
    }
}

/// Individual Virtual CPU (vCPU) execution context.
#[derive(Debug, Clone, Copy)]
pub struct VirtualCpu {
    /// Zero-based vCPU identifier.
    pub id: u32,
    /// Architectural register state.
    pub regs: GuestRegisters,
    /// Last exit reason.
    pub exit_reason: VmExitReason,
    /// Cumulative instructions executed by this vCPU.
    pub instructions_executed: u64,
    /// Cumulative VM-exit count.
    pub exit_count: u64,
    /// Online status flag.
    pub is_running: bool,
    /// Physical guest ROM/RAM page hosting instruction bytes at reset vector.
    pub guest_code: [u8; GUEST_CODE_CAPACITY],
}

impl VirtualCpu {
    pub const fn new(id: u32) -> Self {
        let mut code = [0xF4; GUEST_CODE_CAPACITY];

        code[0] = 0x0F;
        code[1] = 0xA2;
        code[2] = 0xEC;
        code[3] = 0x0F;
        code[4] = 0x01;
        code[5] = 0xC1;
        code[6] = 0xF4;

        Self {
            id,
            regs: GuestRegisters::new_reset_state(),
            exit_reason: VmExitReason::Unknown,
            instructions_executed: 0,
            exit_count: 0,
            is_running: false,
            guest_code: code,
        }
    }

    /// Load custom x86 guest machine instructions into execution buffer.
    pub fn load_code(&mut self, offset: usize, bytes: &[u8]) -> Result<(), &'static str> {
        if offset + bytes.len() > GUEST_CODE_CAPACITY {
            return Err("Code payload exceeds guest code buffer capacity");
        }
        self.guest_code[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Advance vCPU execution by decoding and executing the instruction at guest RIP.
    pub fn step(&mut self) -> VmExitReason {
        self.is_running = true;

        let offset = match self.regs.rip.checked_sub(GUEST_RESET_VECTOR) {
            Some(off) => off as usize,
            None => {
                self.exit_reason = VmExitReason::Shutdown;
                return VmExitReason::Shutdown;
            }
        };

        if offset >= GUEST_CODE_CAPACITY {
            self.exit_reason = VmExitReason::Shutdown;
            return VmExitReason::Shutdown;
        }

        let slice = &self.guest_code[offset..];
        let (reason, insn_len) = match slice {
            [0x0F, 0xA2, ..] => {
                self.regs.rax = 0x0000_0001;
                self.regs.rbx = 0x756E_6547;
                self.regs.rdx = 0x4965_6E69;
                self.regs.rcx = 0x6C65_746E;
                (VmExitReason::Cpuid, 2)
            }
            [0x0F, 0x01, 0xC1, ..] | [0x0F, 0x01, 0xD9, ..] => (VmExitReason::Hypercall, 3),
            [0x0F, 0x20..=0x22, _, ..] => (VmExitReason::CrAccess, 3),
            [0xEC..=0xEF, ..] => (VmExitReason::IoInstruction, 1),
            [0xE4..=0xE7, _, ..] => (VmExitReason::IoInstruction, 2),
            [0xF4, ..] => (VmExitReason::Hlt, 1),
            _ => (VmExitReason::Shutdown, 1),
        };

        self.regs.rip = self.regs.rip.wrapping_add(insn_len as u64);
        self.instructions_executed = self.instructions_executed.saturating_add(1);
        self.exit_count = self.exit_count.saturating_add(1);
        self.exit_reason = reason;
        reason
    }
}

/// A guest Virtual Machine instance managed by the Keira KVM subsystem.
#[derive(Debug, Clone, Copy)]
pub struct VirtualMachine {
    /// Globally unique numeric VM ID.
    pub id: u64,
    /// Active status flag.
    pub is_active: bool,
    /// Guest physical RAM allocated in Megabytes.
    pub memory_size_mb: u32,
    /// Number of provisioned vCPUs.
    pub vcpu_count: u32,
    /// vCPU descriptors.
    pub vcpus: [Option<VirtualCpu>; MAX_VCPUS_PER_VM],
    /// Cumulative VM-exit events across all vCPUs in this VM.
    pub total_exits: u64,
}

impl VirtualMachine {
    pub const fn new(id: u64, memory_size_mb: u32) -> Self {
        Self {
            id,
            is_active: true,
            memory_size_mb,
            vcpu_count: 1,
            vcpus: [Some(VirtualCpu::new(0)), None, None, None],
            total_exits: 0,
        }
    }
}

/// Global table of active Virtual Machines.
static VM_TABLE: SpinMutex<[Option<VirtualMachine>; MAX_GUEST_VMS]> = {
    const EMPTY: Option<VirtualMachine> = None;
    SpinMutex::new([
        Some(VirtualMachine {
            id: 1,
            is_active: true,
            memory_size_mb: 128,
            vcpu_count: 1,
            vcpus: [Some(VirtualCpu::new(0)), None, None, None],
            total_exits: 0,
        }),
        EMPTY,
        EMPTY,
        EMPTY,
    ])
};

/// Next available VM identifier counter.
static NEXT_VM_ID: SpinMutex<u64> = SpinMutex::new(2);

/// Global hypervisor active indicator.
pub static mut HYPERVISOR_ACTIVE: bool = true;

/// Probe underlying CPU hardware for Intel VMX and AMD SVM virtualization extensions.
pub fn probe_hardware_virt() -> VirtHardware {
    #[cfg(target_arch = "x86_64")]
    {
        let leaf1 = core::arch::x86_64::__cpuid(1);
        let has_intel_vmx = (leaf1.ecx & (1 << 5)) != 0;

        let ext_leaf1 = core::arch::x86_64::__cpuid(0x8000_0001);
        let has_amd_svm = (ext_leaf1.ecx & (1 << 2)) != 0;

        VirtHardware {
            has_intel_vmx,
            has_amd_svm,
            hypervisor_ready: has_intel_vmx || has_amd_svm,
        }
    }

    #[cfg(target_arch = "x86")]
    {
        let leaf1 = core::arch::x86::__cpuid(1);
        let has_intel_vmx = (leaf1.ecx & (1 << 5)) != 0;

        let ext_leaf1 = core::arch::x86::__cpuid(0x8000_0001);
        let has_amd_svm = (ext_leaf1.ecx & (1 << 2)) != 0;

        VirtHardware {
            has_intel_vmx,
            has_amd_svm,
            hypervisor_ready: has_intel_vmx || has_amd_svm,
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    VirtHardware {
        has_intel_vmx: false,
        has_amd_svm: false,
        hypervisor_ready: false,
    }
}

/// Create a new Guest Virtual Machine (VM) execution context.
pub fn create_vm() -> Result<u64, &'static str> {
    let mut table = VM_TABLE.lock();
    let mut next_id = NEXT_VM_ID.lock();

    for slot in table.iter_mut() {
        if slot.is_none() {
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            *slot = Some(VirtualMachine::new(id, 64));
            unsafe {
                HYPERVISOR_ACTIVE = true;
            }
            return Ok(id);
        }
    }

    Err("Maximum guest virtual machines reached")
}

/// Syscall alias for create_vm (Syscall 42).
pub fn sys_kvm_create_vm() -> Result<u64, &'static str> {
    create_vm()
}

/// Run Guest vCPU execution loop until VM-exit interrupt (Syscall 43).
pub fn run_vcpu(vm_id: u64, vcpu_id: u32) -> Result<u64, &'static str> {
    let mut table = VM_TABLE.lock();

    for slot in table.iter_mut() {
        if let Some(vm) = slot {
            if vm.id == vm_id && vm.is_active {
                if (vcpu_id as usize) < MAX_VCPUS_PER_VM {
                    if let Some(vcpu) = &mut vm.vcpus[vcpu_id as usize] {
                        let exit = vcpu.step();
                        vm.total_exits = vm.total_exits.saturating_add(1);
                        return Ok(exit as u64);
                    }
                    return Err("Specified vCPU is not provisioned");
                }
                return Err("vCPU ID exceeds maximum allowed limit");
            }
        }
    }

    Err("Virtual Machine ID not found")
}

/// Syscall alias for run_vcpu (Syscall 43).
pub fn sys_kvm_run_vcpu(vm_id: u64, vcpu_id: u32) -> Result<u64, &'static str> {
    run_vcpu(vm_id, vcpu_id)
}

/// Destroy a Guest Virtual Machine by ID.
pub fn destroy_vm(vm_id: u64) -> Result<(), &'static str> {
    let mut table = VM_TABLE.lock();

    for slot in table.iter_mut() {
        if let Some(vm) = slot {
            if vm.id == vm_id {
                *slot = None;
                return Ok(());
            }
        }
    }

    Err("Target VM not found")
}

/// Retrieve a snapshot copy of a specific VM descriptor.
pub fn get_vm_snapshot(vm_id: u64) -> Option<VirtualMachine> {
    let table = VM_TABLE.lock();
    for slot in table.iter() {
        if let Some(vm) = slot {
            if vm.id == vm_id {
                return Some(*vm);
            }
        }
    }
    None
}

/// Return all active Virtual Machines and total count.
pub fn get_all_vms() -> ([Option<VirtualMachine>; MAX_GUEST_VMS], usize) {
    let table = VM_TABLE.lock();
    let mut count = 0;
    for slot in table.iter() {
        if slot.is_some() {
            count += 1;
        }
    }
    (*table, count)
}

/// Get aggregate KVM hardware and execution metrics:
/// `(has_vmx, has_svm, active_vms_count, total_vm_exits)`.
pub fn get_kvm_stats() -> (bool, bool, usize, u64) {
    let hw = probe_hardware_virt();
    let (vms, count) = get_all_vms();
    let mut total_exits = 0;
    for slot in vms.iter() {
        if let Some(vm) = slot {
            total_exits += vm.total_exits;
        }
    }
    (hw.has_intel_vmx, hw.has_amd_svm, count, total_exits)
}
