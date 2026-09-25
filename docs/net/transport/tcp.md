<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Transmission Control Protocol (TCP) State Machine

Keira implements a stateful, RFC 793 compliant TCP protocol engine (`crates/net/src/tcp/`) supporting reliable, ordered stream communication.

---

## 1. TCP Connection State Machine

```mermaid
stateDiagram-v2
    [*] --> CLOSED
    CLOSED --> LISTEN: Passive Open (bind/listen)
    CLOSED --> SYN_SENT: Active Open (connect, send SYN)
    LISTEN --> SYN_RCVD: Recv SYN, send SYN-ACK
    SYN_SENT --> ESTABLISHED: Recv SYN-ACK, send ACK
    SYN_RCVD --> ESTABLISHED: Recv ACK
    ESTABLISHED --> FIN_WAIT_1: Active Close (close, send FIN)
    ESTABLISHED --> CLOSE_WAIT: Passive Close (recv FIN, send ACK)
    FIN_WAIT_1 --> FIN_WAIT_2: Recv ACK of FIN
    FIN_WAIT_2 --> TIME_WAIT: Recv FIN, send ACK
    CLOSE_WAIT --> LAST_ACK: Close call, send FIN
    LAST_ACK --> CLOSED: Recv ACK of FIN
    TIME_WAIT --> CLOSED: 2 * MSL Timeout
```

---

## 2. Sliding Window & Flow Control

* **Sequence Tracking**: Every connection tracks `snd_una` (unacknowledged send), `snd_nxt` (next send sequence), and `rcv_nxt` (expected receive sequence).
* **Receive Window (`rcv_wnd`)**: Advertised in every TCP header to throttle remote transmitters according to available ring buffer space.
