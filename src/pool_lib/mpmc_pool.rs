use crate::pool_lib::{Message, ThreadPool, BasicThreadPool, Worker, Sender, Receiver};
use crossbeam::channel;
use std::thread;
use std::thread::JoinHandle;
use std::error::Error;
type MPMCReceiver = channel::Receiver<Message>;
struct MPMCWorker {
    id: usize,
    // 里面的线程是可为空的
    thread: Option<thread::JoinHandle<()>>,
}

impl Receiver for MPMCReceiver {
    fn recv(&self) -> Result<Message, Box<dyn Error>> {
        let message = self.recv()?;
        Ok(message)
    }
}
impl Sender for channel::Sender<Message> {
    fn send(&self, message: Message) -> Result<(), Box<dyn Error>> {
        self.send(message)?;
        Ok(())
    }
}
impl Worker for MPMCWorker {
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
impl MPMCWorker {
    fn new(id: usize, receiver: MPMCReceiver) -> Self {
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
    workers: Vec<MPMCWorker>,
}
impl MPMCThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        // 创建通道
        let (sender, receiver) = channel::unbounded();
        Self {
            workers: (0..size)
                .map(|i| MPMCWorker::new(i, receiver.clone()))
                .collect::<_>(),
            sender, // 发送者
        }
    }
}
impl BasicThreadPool for MPMCThreadPool {
    fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(box f);
        self.sender.send(message).unwrap();
    }
}
impl ThreadPool<MPMCWorker, channel::Sender<Message>> for MPMCThreadPool {
    fn workers_mut(&mut self) -> &mut Vec<MPMCWorker> {
        &mut self.workers
    }

    fn workers(&self) -> &Vec<MPMCWorker> {
        &self.workers
    }

    fn sender(&self) -> &channel::Sender<Message> {
        &self.sender
    }
}

impl Drop for MPMCThreadPool {
    fn drop(&mut self) {
        ThreadPool::drop(self)
    }
}
