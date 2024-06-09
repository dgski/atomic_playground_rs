# Atomic Playground (Rust)

Strengthening my atomic operation knowledge with some hands-on coding.

## Components

- [WaitFreeChannel](./src/waitfree_channel.rs): Inter-thread communication 'channel' with lock and Wait free reading, lock-free writing. Results on M1 Macbook Air for 10s max reading/writing of 32 byte:
    - Average time to read: 53 ns, total reads: 102,972,823
    - Average time to write: 48 ns, total writes: 118,251,756