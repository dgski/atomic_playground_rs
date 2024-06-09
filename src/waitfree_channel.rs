use std::sync::atomic::{AtomicPtr, Ordering};

/// A wait-free reusable channel that can be used to send and receive values between threads.
pub struct AtomicChannel<T> {
    values: [Option<T>; 2],
    index: usize,
    ptr: AtomicPtr<T>
}

unsafe impl <T> Send for AtomicChannel<T> {}

impl<T> AtomicChannel<T> {
    /// Creates a new `AtomicChannel`.
    pub fn new() -> Self {
        AtomicChannel {
            values: [None, None],
            index: 0,
            ptr: AtomicPtr::new(std::ptr::null_mut())
        }
    }

    /// Returns a reader for the channel.
    pub fn get_reader(&mut self) -> AtomicChannelReader<T> {
        AtomicChannelReader {
            channel: self
        }
    }
    
    /// Returns a writer for the channel.
    pub fn get_writer(&mut self) -> AtomicChannelWriter<T> {
        AtomicChannelWriter {
            channel: self
        }
    }
}

/// A reader for the `AtomicChannel`.
pub struct AtomicChannelReader<T> {
    channel: *mut AtomicChannel<T>
}

impl <T> AtomicChannelReader<T> {
    /// Receives a value from the channel.
    /// Returns a reference to the value.
    /// If the channel is empty, the function will return `None`.
    /// The value will be removed from the channel.
    pub fn recv(&self) -> Option<&mut T> {
        let channel = unsafe{ &mut *self.channel };
        let val = channel.ptr.swap(std::ptr::null_mut(), Ordering::Acquire);
        if val.is_null() {
            return None;
        }
        unsafe{ Some(&mut *(val)) }
    }
}

unsafe impl<T> Send for AtomicChannelReader<T> {}

/// A writer for the `AtomicChannel`.
pub struct AtomicChannelWriter<T> {
    channel: *mut AtomicChannel<T>
}

impl <T> AtomicChannelWriter<T> {
    /// Sends a value to the channel.
    /// Returns `true` if the value was sent successfully, `false` otherwise.
    /// If the value was not sent successfully, it means that the channel is full.
    /// In this case, the value is not sent and the function returns `false`.
    pub fn send(&self, value: T) -> bool {
        let channel = unsafe{ &mut *self.channel };
        if !channel.ptr.load(Ordering::Acquire).is_null() {
            return false;
        }
        channel.values[channel.index] = Some(value);
        channel.ptr.store(channel.values[channel.index].as_mut().unwrap(), Ordering::Release);
        channel.index = (channel.index + 1) % 2;
        return true;
    }
}

unsafe impl<T> Send for AtomicChannelWriter<T> {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use super::*;

    #[test]
    fn basics() {
        let mut channel = AtomicChannel::<i32>::new();
        let reader = channel.get_reader();
        let writer = channel.get_writer();

        assert_eq!(writer.send(123), true);
        assert_eq!(reader.recv(), Some(&mut 123));
        assert_eq!(reader.recv(), None);
    }

    #[test]
    fn benchmark() {
        let running = AtomicBool::new(true);
        let mut channel = AtomicChannel::<i32>::new();

        let reader = channel.get_reader();
        let reader_running = unsafe{ AtomicBool::from_ptr(running.as_ptr()) };
        let t1 = std::thread::spawn(move || {
            let mut total_nanoseconds_reading = 0;
            let mut total_reads = 0;
            while reader_running.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let _result = reader.recv();
                let duration = start.elapsed();
                total_nanoseconds_reading += duration.as_nanos();
                total_reads += 1;
            }
            (total_nanoseconds_reading, total_reads)
        });

        let writer = channel.get_writer();
        let writer_running = unsafe{ AtomicBool::from_ptr(running.as_ptr()) };
        let t2 = std::thread::spawn(move || {
            let mut total_nanoseconds_writing = 0;
            let mut total_writes = 0;
            while writer_running.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let _result = writer.send(112341234);
                let duration = start.elapsed();
                total_nanoseconds_writing += duration.as_nanos();
                total_writes += 1;
            }
            (total_nanoseconds_writing, total_writes)
        });

        std::thread::sleep(std::time::Duration::from_secs(10));

        running.store(false, Ordering::Relaxed);

        let (total_nanoseconds_reading, total_reads) = t1.join().unwrap();
        let (total_nanoseconds_writing, total_writes) = t2.join().unwrap();

        // Print the results
        println!("Average time to read: {} ns, total reads: {}", total_nanoseconds_reading / total_reads, total_reads);
        println!("Average time to write: {} ns, total writes: {}", total_nanoseconds_writing / total_writes, total_writes);

    }
}