use super::{Message,Worker};
use crate::pool_lib::ThreadPool;
use std::sync::{mpsc, Arc, Mutex}; // 生产者消费者模型
use std::thread;
use std::thread::JoinHandle;
// 接收者，使用了引用计数和互斥锁来保证多所有者共享和互斥访问
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;

/// 线程池
pub struct NaiveThreadPool {
    /// 工作线程
    workers: Vec<NaiveWorker>,
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
                .map(|i| NaiveWorker::new(i, Arc::clone(&receiver)))
                .collect::<_>(),
            sender, // 发送者
        }
    }
}
impl ThreadPool for NaiveThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(Box::new(f));
        self.sender.send(message).unwrap();
    }

    fn workers(&self) -> Vec<Box<dyn Worker>> {
        Box::new(self.workers)
    }
}

/// 停机处理
impl Drop for NaiveThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        // 先发送停机message
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");
        // 等待所有worker关闭
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            // 利用take将线程从worker中取出
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
struct NaiveWorker {
    id: usize,
    // 里面的线程是可为空的
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker for NaiveWorker {
    fn thread(&self) -> Box<Option<JoinHandle<()>>> {
        Box::new(self.thread)
    }

    fn id(&self) -> usize {
        self.id
    }
}
impl NaiveWorker {
    fn new(id: usize, receiver: Receiver) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || {
                // 不断尝试获得锁并读取message
                loop {
                    match receiver.lock().unwrap().recv().unwrap() {
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
