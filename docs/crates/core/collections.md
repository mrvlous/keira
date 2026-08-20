<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Core Collections & Ring Buffers

Documentation for the fixed-capacity collections provided by [`crates/core/src/collections/`](../../../crates/core/src/collections).

## 1. `RingBuffer<T, N>`

Fixed-capacity circular FIFO buffer designed for interrupt-safe serial transmission, keyboard scan code buffering, and lock-free queuing.

### API Reference
- `new() -> Self`: Initializes an empty ring buffer.
- `push(item: T) -> Result<(), T>`: Inserts an item at the head index. Returns `Err(item)` if full.
- `pop() -> Option<T>`: Removes and returns the item at the tail index.
- `is_empty() -> bool`: Returns `true` if head equals tail.
- `is_full() -> bool`: Returns `true` if next head position equals tail.
- `len() -> usize`: Calculates active element count.

```rust
use keira_core::collections::RingBuffer;

let mut rx_queue: RingBuffer<u8, 256> = RingBuffer::new();
rx_queue.push(0x41).expect("Buffer overflow");
assert_eq!(rx_queue.pop(), Some(0x41));
```

## 2. `LruCache<K, V, N>`

Static Least-Recently-Used (LRU) cache table used by filesystem sector caches (e.g. FAT sector lookup) and network resolution tables.

### API Reference
- `get(&mut self, key: &K) -> Option<&V>`: Retrieves a cached entry and updates its access age.
- `insert(&mut self, key: K, value: V)`: Inserts a new entry, automatically evicting the oldest element if capacity `N` is reached.
