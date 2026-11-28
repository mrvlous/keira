.. SPDX-License-Identifier: GPL-2.0-only


Milestone 69: End-to-End RFC 8446 TLS 1.3 & HTTPS Transport Architecture
========================================================================

Milestone 69 upgrades Keira's bare-metal networking subsystem into a complete dual HTTP and HTTPS client/transport engine built from first principles without OpenSSL or external TLS dependencies. Operating cleanly over Keira's Ring 0 stateful TCP engine (``crates/net/src/tcp/``) and exposed through the ``SYS_TLS_CONNECT`` system call (Vector 33), unprivileged Ring 3 binaries such as ``fetch`` and ``curl`` can fetch encrypted REST APIs and follow HTTP-to-HTTPS redirects seamlessly.

1. Sequence-Ordered TCP Stream Transport (``crates/net/src/tcp/stream.rs``)
=============================================================================

Reliable TLS 1.3 record parsing requires strictly ordered byte delivery without drops or duplicates:

* **MSS-Bounded TX Segmentation**: ``tcp_send_chunked()`` splits outbound payloads at ``TCP_MSS = 1400`` bytes while updating ``snd_nxt`` monotonically.
* **Sequence-Ordered RX Reassembly**: ``tcp_receive_stream()`` enforces exact byte matching against ``rcv_nxt``, ACKs duplicate retransmissions cleanly and drains 25ms NIC bursts while preserving buffered tail bytes across read calls.

2. RFC 8446 TLS 1.3 Cryptographic Handshake (``crates/net/src/tls/``)
======================================================================

Built atop Ring 0 primitives in ``crates/crypto`` (``x25519``, ``aes128_gcm``, ``sha256``, ``hmac_sha256``) and hardware entropy (``get_entropy_u64()``):

1. **ClientHello Negotiation**: Emits ephemeral X25519 ``key_share`` (``0x0033``), Server Name Indication (SNI), ``supported_versions`` TLS 1.3 (``0x002B``), ``supported_groups`` X25519 (``0x000A``) and ``signature_algorithms`` (``0x000D``).
2. **HKDF-SHA256 Key Schedule (``keyschedule.rs``)**: Derives ``early_secret``, ``handshake_secret``, ``c_hs_traffic``/``s_hs_traffic``, ``master_secret`` and ``c_ap_traffic``/``s_ap_traffic`` along with per-record AEAD key and 12-byte XOR-sequence IVs.
3. **Encrypted Handshake & X.509 Verification (``x509.rs``)**: Decrypts ``EncryptedExtensions``, ``Certificate``, ``CertificateVerify`` and ``Finished``, validates X.509 Subject Alternative Name (``OID 2.5.29.17``) and Common Name (``OID 2.5.4.3``) wildcard patterns unless ``TLS_FLAG_INSECURE`` is requested and transmits the client ``Finished`` MAC.
4. **Socket Transparency**: ``SocketEntry::tls_slot`` routes ``send()``, ``recv()`` and ``close()`` through AEAD record encryption (``ContentType 0x17``) and ``close_notify`` teardown automatically.

3. Freestanding HTTPS Client Upgrades (``userland/netutils/fetch.c``)
=====================================================================

* **Protocol Selection**: Parses both ``http://`` (port 80) and ``https://`` (port 443) URLs, invoking ``sys_tls_attach(sockfd, host, flags)`` after TCP connect.
* **Curl Semantics**: Adds ``-k``/``--insecure`` TLS override, ``-L``/``--location`` 301/302/307/308 redirect traversal (including cross-scheme HTTP-to-HTTPS hops), HTTP ``Transfer-Encoding: chunked`` decoding and ``-v`` TLS handshake diagnostics on stderr.
