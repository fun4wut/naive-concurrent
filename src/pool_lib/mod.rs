mod mpmc_pool;
mod naive_pool;
pub use mpmc_pool::MPMCThreadPool;
pub use naive_pool::NaiveThreadPool;
pub use scheduled_thread_pool::ScheduledThreadPool;
use std::thread::JoinHandle;
use std::error::Error;
use std::thread;
// 要传递的闭包，Send来线程间传递，'static生命周期意味着贯穿整个程序，因为不知道该线程执行多久
type Job = Box<dyn FnOnce() + Send + 'static>;
/// 传递的信息，有可能是新的任务，或是终止信息
enum Message {
    NewJob(Job),
    Terminate,
}
trait Sender {
    fn send(&self, message: Message) -> Result<(), Box<dyn Error>>;
}
trait Receiver {
    fn recv(&self) -> Result<Message, Box<dyn Error>>;
}
// Worker的trait
trait Worker {
    fn thread(&self) -> &Option<JoinHandle<()>>;
    fn thread_mut(&mut self) -> &mut Option<JoinHandle<()>>;
    fn id(&self) -> usize;
    fn init(&mut self) -> JoinHandle<()> {
        let thread = thread::spawn(move || {
            // 不断尝试获得锁并读取message
            loop {
                match receiver.lock().unwrap().recv().unwrap() {
                    // 收到任务消息，执行任务
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", self.id());
                        job();
                    }
                    // 收到终止消息，结束loop
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", self.id());
                        break;
                    }
                }
            }
        };
    }
}
/// 基本的threadpool类型，只需要execute方法
pub trait BasicThreadPool {
    fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static;
}
/// 自定义pool需满足的trait
/// 使用泛型来约束（关联类型亦可）
trait ThreadPool<T: Worker, S: Sender>: Drop + BasicThreadPool {
    // 返回两种引用，让borrow checker满意
    fn workers_mut(&mut self) -> &mut Vec<T>;
    fn workers(&self) -> &Vec<T>;
    fn sender(&self) -> &S;
    // 没法impl trait for trait,所以需要曲线救国
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        // 先发送停机message
        for _ in self.workers() {
            self.sender().send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");
        // 等待所有worker关闭
        for worker in self.workers_mut() {
            println!("Shutting down worker {}", worker.id());
            // 利用take将线程从worker中取出
            if let Some(thread) = worker.thread_mut().take() {
                thread.join().unwrap();
            }
        }
    }
}

/// 使第三方crate获得ThreadPool接口
impl BasicThreadPool for ScheduledThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.execute(f);
    }
}
