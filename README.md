# Atomic Playground (Rust)

Strengthening my atomic operation knowledge with some hands-on coding.

## Components

- [WaitFreeChannel](./src/waitfree_channel.rs): Inter-thread communication 'channel' with lock and Wait free reading, lock-free writing. Results on M1 Macbook Air for 10s max reading/writing of 32 byte:
    - Average time to read: 57 ns, total reads: 116,812,897.
    - Average time to write: 29 ns, total writes: 172,497,080.
- [WaitFreeQueue](./src/waitfree_queue.rs): Inter-thread queue with lock and wait free reading, lock-free writing. Results on M1 Macbook Air:
    - Average time to read: 41 ns, total reads: 145,907,643.
    - Average time to write: 41 ns, total writes: 145,698,898.
- [WaitFreeWorker](./src/waitfree_worker.rs): Single background thread worker that leverages wait-free queues to allow rapid scheduling of tasks and subsequent callbacks. Results on M1 Macbook Air:
    - Average posting time: 78 ns.
    - Average polling time: 31 ns.