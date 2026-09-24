// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for packet firewall rules and BPF runtime engine.

#[cfg(test)]
mod test {
    use super::super::bpf::*;
    use super::super::firewall::*;

    #[test]
    fn test_firewall_rules_and_filtering() {
        flush_rules();
        unsafe {
            NETFILTER_ENABLED = true;
        }

        let idx = add_rule("INPUT", "TCP", 8080, "DROP", "0.0.0.0/0", "0.0.0.0/0")
            .expect("add rule drop 8080");
        assert_eq!(idx, 1);

        let mut frame = [0u8; 54];
        frame[12] = 0x08;
        frame[13] = 0x00;
        frame[23] = 6;
        frame[36] = (8080 >> 8) as u8;
        frame[37] = (8080 & 0xff) as u8;

        unsafe {
            assert!(!filter_ipv4_frame(&frame));
        }

        frame[36] = 0;
        frame[37] = 80;
        unsafe {
            assert!(filter_ipv4_frame(&frame));
        }

        assert!(delete_rule(idx).is_ok());
        assert!(delete_rule(idx).is_err());

        frame[36] = (8080 >> 8) as u8;
        frame[37] = (8080 & 0xff) as u8;
        unsafe {
            assert!(filter_ipv4_frame(&frame));
        }

        flush_rules();
    }

    #[test]
    fn test_bpf_verifier() {
        let valid_prog = [
            BpfInstruction {
                code: BPF_LD | BPF_IMM,
                jt: 0,
                jf: 0,
                k: 42,
            },
            BpfInstruction {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: 0xFFFF,
            },
        ];
        assert!(bpf_verify(&valid_prog).is_ok());

        let no_ret_prog = [BpfInstruction {
            code: BPF_LD | BPF_IMM,
            jt: 0,
            jf: 0,
            k: 42,
        }];
        assert!(bpf_verify(&no_ret_prog).is_err());

        let div_zero_prog = [
            BpfInstruction {
                code: BPF_ALU | BPF_DIV | BPF_K,
                jt: 0,
                jf: 0,
                k: 0,
            },
            BpfInstruction {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: 0xFFFF,
            },
        ];
        assert!(bpf_verify(&div_zero_prog).is_err());

        let oob_jmp_prog = [
            BpfInstruction {
                code: BPF_JMP | BPF_JA,
                jt: 0,
                jf: 0,
                k: 5,
            },
            BpfInstruction {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: 0,
            },
        ];
        assert!(bpf_verify(&oob_jmp_prog).is_err());
    }

    #[test]
    fn test_bpf_vm_execution() {
        let insns = [
            BpfInstruction {
                code: BPF_LD | BPF_IMM,
                jt: 0,
                jf: 0,
                k: 10,
            },
            BpfInstruction {
                code: BPF_ALU | BPF_ADD | BPF_K,
                jt: 0,
                jf: 0,
                k: 32,
            },
            BpfInstruction {
                code: BPF_RET | BPF_A,
                jt: 0,
                jf: 0,
                k: 0,
            },
        ];
        let packet = [0u8; 16];
        let ret = bpf_run_filter(&insns, &packet);
        assert_eq!(ret, 42);
    }

    #[test]
    fn test_bpf_map_operations() {
        let map_id = 99;
        unsafe {
            create_map_internal(7, map_id, BpfMapType::Hash, "test_map", 4, 8, 16);
        }

        assert_eq!(map_lookup(map_id, 100), None);
        assert!(map_update(map_id, 100, 555).is_ok());
        assert_eq!(map_lookup(map_id, 100), Some(555));

        assert!(map_update(map_id, 100, 777).is_ok());
        assert_eq!(map_lookup(map_id, 100), Some(777));

        assert!(map_delete(map_id, 100).is_ok());
        assert_eq!(map_lookup(map_id, 100), None);
        assert!(map_delete(map_id, 100).is_err());
    }
}
