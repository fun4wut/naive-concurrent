use super::Message;
use crate::pool_lib::ThreadPool;
use std::sync::{mpsc, Arc, Mutex}; // 生产者消费者模型
use std::thread;

// 接收者，使用了引用计数和互斥锁来保证多所有者共享和互斥访问
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;

/// 线程池
pub struct NaiveThreadPool {
    /// 工作线程
    workers: Vec<Worker>,
    /// 信息的发送者
    sender: mpsc::Sender<Message>,
}
impl NaiveThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        // 创建通道
        let (sender, receiver) = mpsc::channel();
        // 包装一下接收者
        let receiver = Arc::new(Mutex::new(receiver));
        Self {
            workers: (0..size)
                .map(|i| Worker::new(i, Arc::clone(&receiver)))
                .collect::<_>(),
            sender, // 发送者
        }
    }
}
impl_pool_traits! {NaiveThreadPool}

gen_struct! {worker}

impl Worker {
    fn new(id: usize, receiver: Receiver) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || {
                // 不断尝试获得锁并读取message
                loop {
                    let message = receiver.lock().unwrap().recv().unwrap();
                    match message {
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
