use crate::pool_lib::{Message, ThreadPool};
use crossbeam::channel;
use crossbeam::channel::Receiver;
use std::thread;
gen_struct! {worker}
impl Worker {
    fn new(id: usize, receiver: Receiver<Message>) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || {
                // 不断尝试获得锁并读取message
                loop {
                    match receiver.recv().unwrap() {
                        // 收到任务消息，执行任务
                        Message::NewJob(job) => {
                            println!("Worker {} got a job; executing.", id);
                            job();
                        }
                        // 收到终止消息，结束loop
                        Message::Terminate => {
                            println!("Worker {} was told to terminate.", id);
                            break;
                        }
                    }
                }
            })),
        }
    }
}
pub struct MPMCThreadPool {
    sender: channel::Sender<Message>,
    workers: Vec<Worker>,
}
impl MPMCThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        // 创建通道
        let (sender, receiver) = channel::unbounded();
        Self {
            workers: (0..size)
                .map(|i| Worker::new(i, receiver.clone()))
                .collect::<_>(),
            sender, // 发送者
        }
    }
}
impl_pool_traits! {MPMCThreadPool}
