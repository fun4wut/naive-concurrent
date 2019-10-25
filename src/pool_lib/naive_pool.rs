use super::{Message,Worker};
use crate::pool_lib::{ThreadPool, BasicThreadPool, Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex}; // 生产者消费者模型
use std::thread;
use std::thread::JoinHandle;
use std::error::Error;

// 接收者，使用了引用计数和互斥锁来保证多所有者共享和互斥访问
type NaiveReceiver = Arc<Mutex<mpsc::Receiver<Message>>>;

impl Receiver for NaiveReceiver {
    fn recv(&self) -> Result<Message, Box<dyn Error>> {
        let message = self.lock()?.recv()?;
        Ok(message)
    }
}
impl Sender for mpsc::Sender<Message> {
    fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        self.send(message)?;
        Ok(())
    }
}
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
impl BasicThreadPool for NaiveThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(Box::new(f));
        self.sender.send(message).unwrap();
    }
}
impl ThreadPool<NaiveWorker, mpsc::Sender<Message>> for NaiveThreadPool {
    fn workers_mut(&mut self) -> &mut Vec<NaiveWorker> {
        &mut self.workers
    }

    fn workers(&self) -> &Vec<NaiveWorker> {
        &self.workers
    }

    fn sender(&self) -> &mpsc::Sender<Message> {
        &self.sender
    }
}
/// 停机处理
impl Drop for NaiveThreadPool {
    fn drop(&mut self) {
        ThreadPool::drop(self)
    }
}
struct NaiveWorker {
    id: usize,
    // 里面的线程是可为空的
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker for NaiveWorker {
    fn thread(&self) -> &Option<JoinHandle<()>> {
        &self.thread
    }

    fn thread_mut(&mut self) -> &mut Option<JoinHandle<()>> {
        &mut self.thread
    }

    fn id(&self) -> usize {
        self.id
    }
}
impl NaiveWorker {
    fn new(id: usize, receiver: NaiveReceiver) -> Self {
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
