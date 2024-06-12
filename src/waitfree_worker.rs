use std::{borrow::{Borrow, BorrowMut}, mem, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::sleep};

use crate::waitfree_queue::*;

pub struct WaitFreeWorker {
    task_queue: Box<WaitFreeQueue<Box<dyn FnOnce()>, 1024>>,
    callback_queue: Box<WaitFreeQueue<Box<dyn FnOnce()>, 1024>>,
    running: Box<AtomicBool>,
    thread: std::thread::JoinHandle<()>
}

impl WaitFreeWorker {
    /// Creates a new `WaitFreeWorker`.
    pub fn new() -> Self {
        let mut running = Box::new(AtomicBool::new(true));
        let mut running_sneaker = ThreadSneaker::new(&mut *running);
        let mut task_queue = Box::new(WaitFreeQueue::<Box<dyn FnOnce()>, 1024>::new());
        let mut task_queue_sneaker = ThreadSneaker::new(&mut task_queue);
        let callback_queue = Box::new(WaitFreeQueue::<Box<dyn FnOnce()>, 1024>::new());
        let thread = std::thread::spawn(move || {
            let task_queue = task_queue_sneaker.get_item();
            while running_sneaker.get_item().load(Ordering::Relaxed) {
                let mut task = task_queue.try_read();
                if task.is_none() {
                    continue;
                }
                let mut tmp : Box<dyn FnOnce()> = Box::new(|| {});
                mem::swap(task.unwrap(), &mut tmp);
                tmp();
            }
        });
        WaitFreeWorker {
            task_queue,
            callback_queue,
            running,
            thread
        }
    }

    pub fn schedule_task<T: 'static>(&mut self, task: Box<dyn FnOnce() -> T>, callback: Box<dyn FnOnce(T)>) -> bool {
        let mut callback_queue_sneaker = ThreadSneaker::new(&mut self.callback_queue);
        let success = self.task_queue.try_write(Box::new(move || {
            let result = task();
            callback_queue_sneaker.get_item().try_write(Box::new(move || {
                callback(result);
            }));
        }));
        success
    }

    pub fn run_callbacks(&mut self) {
        let mut count = 0;
        loop {
            let mut callback_queue = self.callback_queue.try_read();
            if callback_queue.is_none() {
                return;
            }
            let mut tmp : Box<dyn FnOnce()> = Box::new(|| {});
            mem::swap(callback_queue.unwrap(), &mut tmp);
            tmp();
            count += 1;
        }
    }
}

impl Drop for WaitFreeWorker {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;

    #[test]
    fn test_wait_free_worker() {
        let mut worker = WaitFreeWorker::new();
        println!("Worker created");
        worker.schedule_task(Box::new(move || {
            println!("Hello, world!");
        }), Box::new(move |_| {
            println!("Callback!");
        }));
        // Sleep for a bit to allow the worker to run
        std::thread::sleep(std::time::Duration::from_secs(5));
        worker.running.store(false, Ordering::Relaxed);
    }

    #[test]
    fn benchmark_worker() {
        let mut unoptimize = 0;
        const ITERATIONS : u128 = 1000000;
        let mut total_posting_nanoseconds = 0;
        let mut total_polling_nanoseconds = 0;
        let mut worker = WaitFreeWorker::new();
        for _ in 0..ITERATIONS {
            {
                let mut unoptimize_sneaker = ThreadSneaker::new(&mut unoptimize);
                let t1 = std::time::Instant::now();
                worker.schedule_task(Box::new(move || {
                    // Create big vector to simulate work
                    let mut v = Vec::new();
                    for i in 0..1000000 {
                        v.push(i);
                    }
                    v
                }), Box::new(move |v| {
                    *(unoptimize_sneaker.get_item()) = v.len();
                }));
                let t2 = std::time::Instant::now();
                total_posting_nanoseconds += t2.duration_since(t1).as_nanos();
            }
            {
                let t1 = std::time::Instant::now();
                worker.run_callbacks();
                let t2 = std::time::Instant::now();
                total_polling_nanoseconds += t2.duration_since(t1).as_nanos();
            }
        }

        // Sleep for a bit to allow the worker to run
        std::thread::sleep(std::time::Duration::from_secs(10));

        println!("unoptimize: {}", unoptimize); // <- This is kinda unsafe as the other thread is still running and writing
        println!("average posting time: {} ns", total_posting_nanoseconds / ITERATIONS);
        println!("average polling time: {} ns", total_polling_nanoseconds / ITERATIONS);
    }
}