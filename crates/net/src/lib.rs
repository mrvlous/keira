// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Network stack (Ethernet, ARP, IPv4, ICMP, UDP, DHCP, DNS, TCP, Sockets, TLS 1.3, Netfilter, eBPF).

pub mod arp;
pub mod dhcp;
pub mod dns;
pub mod driver;
pub mod ethernet;
pub mod filter;
pub mod icmp;
pub mod ip;
pub mod socket;
pub mod tcp;
pub mod tls;

pub use arp::table::{
    handle_arp_packet, lookup_mac, send_arp_announcement, update_arp_cache, ArpEntry, ARP_CACHE,
    ARP_CACHE_COUNT,
};
pub use dhcp::client::{dhcp_auto_configure, DhcpConfig, SYSTEM_DHCP};
pub use dns::resolver::{
    encode_qname, print_dns_cache, resolve_domain, DnsCacheEntry, DnsHeader, DNS_CACHE,
    DNS_CACHE_COUNT,
};
pub use driver::e1000::{
    self, init as e1000_init, receive_raw_frame, transmit_raw_frame, E1000RxDesc, E1000TxDesc,
    E1000_FOUND, E1000_IO_BASE, E1000_MAC, E1000_MEM_BASE, PACKETS_RECEIVED, PACKETS_SENT,
};
pub use driver::rtl8139::{self as rtl8139, Rtl8139Device, RTL8139_DEVICE_ID, RTL8139_VENDOR_ID};
pub use driver::virtio_net::{
    self as virtio_net, VirtioNetDevice, VIRTIO_NET_DEVICE_ID, VIRTIO_NET_VENDOR_ID,
};
pub use ethernet::frame::{EthernetHeader, ETHERTYPE_ARP, ETHERTYPE_IPV4, ETHERTYPE_IPV6};
pub use filter as netfilter;
pub use filter::firewall::{
    add_rule, bpf_filter_packet, delete_rule, filter_ipv4_frame, filter_packet, flush_rules,
    print_firewall_status, sys_netfilter, BpfInstruction, ConnTrackEntry, FirewallRule,
    NETFILTER_CMD_ADD_RULE, NETFILTER_CMD_DEL_RULE, NETFILTER_CMD_FLUSH, NETFILTER_CMD_STATUS,
    NETFILTER_CMD_TOGGLE, NETFILTER_ENABLED, PACKETS_DROPPED, PACKETS_INSPECTED,
};
pub use icmp::ping::send_ping;
pub use ip::ipv4::{
    ip_checksum, parse_ipv4_addr, Ipv4Header, IPPROTO_ICMP, IPPROTO_TCP, IPPROTO_UDP,
};
pub use socket::sock::{
    connect_socket, create_socket, validate_socket_port, TcpSocket, AF_INET, AF_INET6, AF_UNIX,
    AF_UNSPEC, SOCK_DGRAM, SOCK_RAW, SOCK_STREAM,
};
pub use tcp::stream::{
    fetch_http, fetch_http_stream, fetch_stream_download, parse_tcp_payload, tcp_checksum,
    tcp_send_and_receive, TcpHeader, TcpState, STREAM_DOWNLOAD_BUFFER, TCP_FLAG_ACK, TCP_FLAG_FIN,
    TCP_FLAG_PSH, TCP_FLAG_RST, TCP_FLAG_SYN,
};
pub use tls::native::{
    fetch_https, fetch_https_stream, tls_connect, TlsSession, TlsState, TLS_AES_128_GCM_SHA256,
    TLS_CONTENT_APPLICATION_DATA, TLS_CONTENT_HANDSHAKE, TLS_VERSION_12, TLS_VERSION_13,
};

#[cfg(target_arch = "x86_64")]
pub const HTTP_USER_AGENT: &str = concat!(
    "KeiraKernel/",
    env!("CARGO_PKG_VERSION"),
    " (Keira-Kernel; Bare-Metal; x86_64) KeiraNet/",
    env!("CARGO_PKG_VERSION")
);

#[cfg(target_arch = "x86")]
pub const HTTP_USER_AGENT: &str = concat!(
    "KeiraKernel/",
    env!("CARGO_PKG_VERSION"),
    " (Keira-Kernel; Bare-Metal; i686) KeiraNet/",
    env!("CARGO_PKG_VERSION")
);

pub use filter::bpf::{
    bpf_run_filter, bpf_verify, get_maps as bpf_get_maps, get_programs as bpf_get_programs,
    get_status as bpf_get_status, map_delete as bpf_map_delete, map_lookup as bpf_map_lookup,
    map_update as bpf_map_update, BpfMap, BpfMapType, BpfProgType, BpfProgram, BpfStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpf_verifier_and_execution() {
        use filter::bpf::*;

        // A valid program: loads byte at offset 0, checks if equal to 42, returns 1 if true, 0 if false
        let prog = [
            BpfInstruction {
                code: BPF_LD | BPF_B | BPF_ABS,
                jt: 0,
                jf: 0,
                k: 0,
            },
            BpfInstruction {
                code: BPF_JMP | BPF_JEQ | BPF_K,
                jt: 0,
                jf: 1,
                k: 42,
            },
            BpfInstruction {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: 1,
            },
            BpfInstruction {
                code: BPF_RET | BPF_K,
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
        use filter::bpf::*;
        init();

        assert!(bpf_map_update(1, 8080, 100).is_ok());
        assert_eq!(bpf_map_lookup(1, 8080), Some(100));

        assert!(bpf_map_delete(1, 8080).is_ok());
        assert_eq!(bpf_map_lookup(1, 8080), None);
    }
}
