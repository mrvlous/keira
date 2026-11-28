.. SPDX-License-Identifier: GPL-2.0-only


Transport Layer Security (TLS 1.3 / RFC 8446)
=============================================

Keira implements a freestanding RFC 8446 TLS 1.3 protocol engine inside ``crates/net/src/tls/`` layered seamlessly above stateful TCP sockets and exposed to Ring 3 via ``SYS_TLS_CONNECT`` (Vector 33).

1. Cryptographic Cipher Suite & Key Exchange
============================================

* **Cipher Suite**: ``TLS_AES_128_GCM_SHA256`` (``0x1301``) powered by ``crates/crypto``.
* **Key Exchange**: Ephemeral X25519 Elliptic Curve Diffie-Hellman (``0x001D``) seeded with hardware ``RDRAND``/``RDSEED`` entropy via ``get_entropy_u64()``.
* **Key Schedule (``keyschedule.rs``)**: Full HKDF-Extract and HKDF-Expand-Label derivation from Early Secret -> Handshake Secret -> Master Secret with 12-byte XOR-sequence nonce generation.

2. X.509 Certificate Hostname Validation
========================================

``crates/net/src/tls/x509.rs`` implements a zero-allocation ASN.1 DER X.509 certificate parser extracting Subject Alternative Name (``OID 2.5.29.17``) DNS extensions and Common Name (``OID 2.5.4.3``) attributes, validating exact matches and single-label DNS wildcards (``*.example.com``). Callers may opt into ``TLS_FLAG_INSECURE`` (``-k``) to bypass certificate hostname verification when testing self-signed endpoints.

3. Transparent Socket Slot Multiplexing
=======================================

Upon handshake completion, ``SocketTable::upgrade_socket_tls()`` binds a slot in ``TLS_SESSIONS`` to ``SocketEntry::tls_slot``. Subsequent ``send()``, ``recv()`` and ``close()`` system calls transparently wrap and unwrap ``TLSInnerPlaintext`` records using ``ContentType 0x17``.
