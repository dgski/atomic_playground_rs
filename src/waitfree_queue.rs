use std::{array, sync::atomic::{AtomicUsize, Ordering}};

/// A wait-free queue that can be used to send and receive values between threads.
pub struct WaitFreeQueue<T, const SIZE: usize> {
    read_sequence: AtomicUsize,
    read_sequence_cached: usize,
    write_sequence: AtomicUsize,
    write_sequence_cached: usize,
    buffer: [Option<T>; SIZE],
}

fn get_next_index(value: usize, max: usize) -> usize {
    if value + 1 == max {
        0
    } else {
        value + 1
    }
}

/// A wait-free queue that can be used to send and receive values between threads.
impl<T, const SIZE: usize> WaitFreeQueue<T, SIZE> {

    fn get_read_index(&mut self, next_write_index: usize) -> usize {
        if self.read_sequence_cached == next_write_index {
            self.read_sequence_cached = self.read_sequence.load(Ordering::Acquire);
        }
        self.read_sequence_cached
    }

    fn get_write_index(&mut self, next_read_index: usize) -> usize {
        if self.write_sequence_cached == next_read_index {
            self.write_sequence_cached = self.write_sequence.load(Ordering::Acquire);
        }
        self.write_sequence_cached
    }

    /// Creates a new `WaitFreeQueue`.
    pub fn new() -> Self {
        WaitFreeQueue {
            buffer: array::from_fn(|_| None),
            read_sequence: AtomicUsize::new(0),
            read_sequence_cached: 0,
            write_sequence: AtomicUsize::new(0),
            write_sequence_cached: 0,
        }
    }

    /// Tries to write a value to the queue.
    pub fn try_write(&mut self, value: T) -> bool {
        let write_index = self.write_sequence.load(Ordering::Relaxed);
        let next_write_index = get_next_index(write_index, SIZE);
        if next_write_index == self.get_read_index(next_write_index) {
            return false;
        }
        self.buffer[write_index] = Some(value);
        self.write_sequence.store(next_write_index, Ordering::Release);
        true
    }

    /// Tries to read a value from the queue.
    pub fn try_read(&mut self) -> Option<&mut T> {
        let read_index = self.read_sequence.load(Ordering::Relaxed);
        if read_index == self.get_write_index(read_index) {
            return None;
        }
        let result = self.buffer[read_index].as_mut().unwrap();
        self.read_sequence.store(get_next_index(read_index, SIZE), Ordering::Release);
        Some(result)
    }
}

/// A helper struct that can 'sneak' a mutable pointer to a value across threads.
pub struct ThreadSneaker<T> {
    item: *mut T
}
unsafe impl<T> Send for ThreadSneaker<T> {}
unsafe impl<T> Sync for ThreadSneaker<T> {}

impl <T> ThreadSneaker<T> {
    /// Creates a new `ThreadSneaker`.
    pub fn new(item: *mut T) -> Self {
        ThreadSneaker {
            item
        }
    }

    /// Returns a mutable reference to the item.
    pub fn get_item(&mut self) -> &mut T {
        unsafe{ &mut *self.item }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;

    #[test]
    fn test_wait_free_queue() {
        let mut queue = WaitFreeQueue::<i32, 64>::new();
        assert_eq!(queue.try_read(), None);
        assert_eq!(queue.try_write(123), true);
        assert_eq!(queue.try_read(), Some(&mut 123));
        assert_eq!(queue.try_read(), None);
    }

    #[test]
    fn benchmark_queue() {
        let running = AtomicBool::new(true);
        let mut queue = WaitFreeQueue::<i64, 65536>::new();

        let reader_running = unsafe{ AtomicBool::from_ptr(running.as_ptr()) };
        let mut reader_sneaker = ThreadSneaker::new(&mut queue);
        let reader_thread = std::thread::spawn(move || {
            let mut total_nanoseconds_reading = 0;
            let mut total_reads = 0;
            let queue = reader_sneaker.get_item();
            while reader_running.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let _result = queue.try_read();
                let duration = start.elapsed();
                total_nanoseconds_reading += duration.as_nanos();
                total_reads += 1;
            }
            (total_nanoseconds_reading, total_reads)
        });

        let writer_running = unsafe{ AtomicBool::from_ptr(running.as_ptr()) };
        let mut writer_sneaker = ThreadSneaker::new(&mut queue);
        let writer_thread = std::thread::spawn(move || {
            let mut total_nanoseconds_writing = 0;
            let mut total_writes = 0;
            let queue = writer_sneaker.get_item();
            while writer_running.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let _result = queue.try_write(112341234);
                let duration = start.elapsed();
                total_nanoseconds_writing += duration.as_nanos();
                total_writes += 1;
            }
            (total_nanoseconds_writing, total_writes)
        });

        std::thread::sleep(std::time::Duration::from_secs(10));
        running.store(false, Ordering::Relaxed);

        let (total_nanoseconds_reading, total_reads) = reader_thread.join().unwrap();
        let (total_nanoseconds_writing, total_writes) = writer_thread.join().unwrap();

        // Print the results
        println!("Average time to read: {} ns, total reads: {}", total_nanoseconds_reading / total_reads, total_reads);
        println!("Average time to write: {} ns, total writes: {}", total_nanoseconds_writing / total_writes, total_writes);
    }
}