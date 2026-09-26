// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Top-level integration and protocol verification tests for keira-net.

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn test_bpf_verifier_and_execution() {
        let prog = [
            BpfInstruction {
                code: filter::bpf::BPF_LD | filter::bpf::BPF_B | filter::bpf::BPF_ABS,
                jt: 0,
                jf: 0,
                k: 0,
            },
            BpfInstruction {
                code: filter::bpf::BPF_JMP | filter::bpf::BPF_JEQ | filter::bpf::BPF_K,
                jt: 0,
                jf: 1,
                k: 42,
            },
            BpfInstruction {
                code: filter::bpf::BPF_RET | filter::bpf::BPF_K,
                jt: 0,
                jf: 0,
                k: 1,
            },
            BpfInstruction {
                code: filter::bpf::BPF_RET | filter::bpf::BPF_K,
                jt: 0,
                jf: 0,
                k: 0,
            },
        ];

        assert!(bpf_verify(&prog).is_ok());

        let pkt_match = [42u8, 1, 2, 3];
        let pkt_no_match = [99u8, 1, 2, 3];

        assert_eq!(bpf_run_filter(&prog, &pkt_match), 1);
        assert_eq!(bpf_run_filter(&prog, &pkt_no_match), 0);
    }

    #[test]
    fn test_bpf_maps() {
        filter::bpf::init();

        assert!(bpf_map_update(1, 8080, 100).is_ok());
        assert_eq!(bpf_map_lookup(1, 8080), Some(100));

        assert!(bpf_map_delete(1, 8080).is_ok());
        assert_eq!(bpf_map_lookup(1, 8080), None);
    }

    #[test]
    fn test_user_agent_format() {
        assert!(HTTP_USER_AGENT.contains("KeiraKernel/"));
        assert!(HTTP_USER_AGENT.contains("x86_64"));
    }
}
