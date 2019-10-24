use std::sync::{mpsc, Arc, Mutex};
use std::thread; // 生产者消费者模型
                 // 接收者，使用了引用计数和互斥锁来保证多所有者共享和互斥访问
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;
// 要传递的闭包，Send来线程间传递，'static生命周期意味着贯穿整个程序，因为不知道该线程执行多久
type Job = Box<dyn FnOnce() + Send + 'static>;
/// 传递的信息，有可能是新的任务，或是终止信息
enum Message {
    NewJob(Job),
    Terminate,
}
/// 线程池
pub struct ThreadPool {
    /// 工作线程
    workers: Vec<Worker>,
    /// 信息的发送者
    sender: mpsc::Sender<Message>,
}
impl ThreadPool {
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
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(box f);
        self.sender.send(message).unwrap();
    }
}

/// 停机处理
impl Drop for ThreadPool {
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
struct Worker {
    id: usize,
    // 里面的线程是可为空的
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
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
