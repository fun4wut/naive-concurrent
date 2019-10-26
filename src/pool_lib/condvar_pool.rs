use crate::pool_lib::{Job, ThreadPool};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

type Notifier = Arc<(Mutex<Status>, Condvar)>;
struct Status {
    queue: VecDeque<Job>,
    shutdown: bool,
}
// 辅助函数，得到下一个任务
fn next_job(notifier: &Notifier) -> Option<Job> {
    // 两层解引用再上一个引用
    let (lock, cvar) = &**notifier;
    // 尝试拿到锁
    let mut status = lock.lock().unwrap();
    loop {
        // 查看队首的任务
        match status.queue.pop_front() {
            // 如果已关机，返回空任务
            None if status.shutdown => return None,
            // 无任务，阻塞当前线程，等待任务的到来
            // wait会自动解开互斥锁（防止死锁)
            None => status = cvar.wait(status).unwrap(),
            // 队列里有任务，返回任务
            some => return some
        }
    }
}
pub struct CVarThreadPool {
    workers: Vec<Worker>,
    notifier: Notifier,
}
impl CVarThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let status = Status {
            queue: VecDeque::new(),
            shutdown: false,
        };
        let notifier = Arc::new((Mutex::new(status), Condvar::new()));
        let mut workers = vec![];
        // 因为所有权的关系，不能使用map闭包
        for i in 0..size {
            let notifier = notifier.clone();
            workers.push(Worker::new(i, notifier));
        }
        Self { notifier, workers }
    }
}

impl ThreadPool for CVarThreadPool {
    fn execute<F>(&self, f: F) where
        F: FnOnce() + Send + 'static
    {
        let (lock, cvar) = &*self.notifier;
        let mut status = lock.lock().unwrap();
        // 队列放入任务
        status.queue.push_back(box f);
        // 唤醒线程
        cvar.notify_one();
    }
}

impl Drop for CVarThreadPool {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.notifier;
        let mut status = lock.lock().unwrap();
        // 设置关闭状态
        status.shutdown = true;
        println!("Sending terminate message to all workers.");
        cvar.notify_one();
        drop(status); // 显式的清除MutexGuard，来退出互斥区
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            // 利用take将线程从worker中取出
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
gen_struct! {worker}
impl Worker {
    fn new(id: usize, notifier: Notifier) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || {
                // 收到任务就继续执行，否则退出循环
                loop {
                    if let Some(job) = next_job(&notifier) {
                        println!("Worker {} got a job; executing.", id);
                        job();
                    } else {
                        break;
                    }
                }
                println!("Worker {} was told to terminate.", id);
            })),
        }
    }
}
