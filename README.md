# Atomic Playground (Rust)

Strengthening my atomic operation knowledge with some hands-on coding.

## Components

- [WaitFreeChannel](./src/waitfree_channel.rs): Inter-thread communication 'channel' with lock and Wait free reading, lock-free writing. Results on M1 Macbook Air for 10s max reading/writing of 32 byte:
- Average time to read: 26 ns, total reads: 161,732,789
- Average time to write: 21 ns, total writes: 173,622,938