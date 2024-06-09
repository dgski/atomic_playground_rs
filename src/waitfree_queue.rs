use std::sync::atomic::{AtomicUsize, Ordering};

/// A wait-free queue that can be used to send and receive values between threads.
pub struct WaitFreeQueue<T: Copy, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    read_sequence: AtomicUsize,
    write_sequence: AtomicUsize
}

/// A wait-free queue that can be used to send and receive values between threads.
impl<T: Copy, const SIZE: usize> WaitFreeQueue<T, SIZE> {
    /// Creates a new `WaitFreeQueue`.
    pub fn new() -> Self {
        WaitFreeQueue {
            buffer: [None; SIZE],
            read_sequence: AtomicUsize::new(0),
            write_sequence: AtomicUsize::new(1)
        }
    }

    /// Tries to write a value to the queue.
    pub fn try_write(&mut self, value: T) -> bool {
        let next_write_index = self.write_sequence.load(Ordering::SeqCst) % SIZE;
        let current_read_index = self.read_sequence.load(Ordering::SeqCst) % SIZE;
        let no_room_left = current_read_index == next_write_index;
        if no_room_left {
            return false;
        }
        self.buffer[next_write_index % SIZE] = Some(value);
        self.write_sequence.store(next_write_index + 1, Ordering::SeqCst);
        true
    }

    /// Tries to read a value from the queue.
    pub fn try_read(&mut self) -> Option<&mut T> {
        let next_read_index = (self.read_sequence.load(Ordering::SeqCst) + 1) % SIZE;
        let next_write_index = self.write_sequence.load(Ordering::SeqCst) % SIZE;
        let no_value_to_read = next_read_index == next_write_index;
        if no_value_to_read {
            return None;
        }
        self.read_sequence.store(next_read_index, Ordering::SeqCst);
        Some(self.buffer[next_read_index].as_mut().unwrap())
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
    fn benchmarks() {
        let running = AtomicBool::new(true);
        let mut queue = WaitFreeQueue::<i64, 64>::new();

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