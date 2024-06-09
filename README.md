# Atomic Playground (Rust)

Strengthening my atomic operation knowledge with some hands-on coding.

## Components

- [WaitFreeChannel](./src/waitfree_channel.rs): Inter-thread communication 'channel' with lock and Wait free reading, lock-free writing. Results on M1 Macbook Air for 10s max reading/writing of 32 byte:
    - Average time to read: 53 ns, total reads: 102,972,823.
    - Average time to write: 48 ns, total writes: 118,251,756.
- [WaitFreeQueue](./src/waitfree_queue.rs): Inter-thread queue with lock and wait free reading, lock-free writing. Results on M1 Macbook Air:
    - Average time to read: 64 ns, total reads: 98,983,966.
    - Average time to write: 65 ns, total writes: 98,141,784.